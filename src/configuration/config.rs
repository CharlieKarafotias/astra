use super::super::constants::{APPLICATION, ORGANIZATION, QUALIFIER};
use super::{
    frequency::Frequency,
    generators::{Generators, JuliaConfig, NasaApodConfig, SolidConfig, SpotlightConfig},
    theme::ThemeConfigs,
    user_config::UserConfig,
};
use directories::ProjectDirs;
use std::{
    error::Error,
    fmt::Display,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub struct Config {
    // true if call to 'astra', false if specific gen called: 'astra generate solid random'
    pub respect_user_config: bool,
    // From CLI options
    verbose: bool,
    user_config: Option<UserConfig>,
}

impl Config {
    pub fn new(verbose: bool) -> Self {
        match Config::read_config_file_if_exists(verbose) {
            Ok(user_config) => Self {
                respect_user_config: false,
                verbose,
                user_config: Some(UserConfig {
                    auto_clean: user_config.auto_clean,
                    frequency: user_config.frequency,
                    generators: user_config.generators,
                    julia_gen: user_config.julia_gen,
                    nasa_apod_gen: user_config.nasa_apod_gen,
                    solid_gen: user_config.solid_gen,
                    spotlight_gen: user_config.spotlight_gen,
                    themes: user_config.themes,
                }),
            },
            Err(e) => {
                if verbose {
                    println!("WARN - ignoring configuration due to error(s): {e}");
                }
                Self {
                    respect_user_config: false,
                    verbose,
                    user_config: None,
                }
            }
        }
    }

    pub fn print_if_verbose(&self, message: &str) {
        if self.verbose {
            println!("{}", message);
        }
    }

    pub fn generators(&self) -> Option<&Generators> {
        if let Some(user_config) = &self.user_config {
            user_config.generators.as_ref()
        } else {
            None
        }
    }

    pub fn frequency(&self) -> Option<&Frequency> {
        if let Some(user_config) = &self.user_config {
            user_config.frequency.as_ref()
        } else {
            None
        }
    }

    pub fn auto_clean(&self) -> Option<&Frequency> {
        if let Some(user_config) = &self.user_config {
            user_config.auto_clean.as_ref()
        } else {
            None
        }
    }

    pub fn solid_gen(&self) -> Option<&SolidConfig> {
        if let Some(user_config) = &self.user_config {
            user_config.solid_gen.as_ref()
        } else {
            None
        }
    }

    pub fn julia_gen(&self) -> Option<&JuliaConfig> {
        if let Some(user_config) = &self.user_config {
            user_config.julia_gen.as_ref()
        } else {
            None
        }
    }

    pub fn nasa_apod_gen(&self) -> Option<&NasaApodConfig> {
        if let Some(user_config) = &self.user_config {
            user_config.nasa_apod_gen.as_ref()
        } else {
            None
        }
    }

    pub fn spotlight_gen(&self) -> Option<&SpotlightConfig> {
        if let Some(user_config) = &self.user_config {
            user_config.spotlight_gen.as_ref()
        } else {
            None
        }
    }

    pub fn themes(&self) -> Option<&ThemeConfigs> {
        if let Some(user_config) = &self.user_config {
            user_config.themes.as_ref()
        } else {
            None
        }
    }

    fn config_dir() -> PathBuf {
        ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
            .map(|dirs| dirs.config_dir().to_path_buf())
            .expect("config folders are defined for each OS")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.json")
    }

    pub fn schema_path() -> PathBuf {
        Self::config_dir().join("config.schema.json")
    }

    /// Renders the configuration as a draft-07 JSON Schema.
    pub fn json_schema() -> String {
        UserConfig::json_schema()
    }

    pub fn create_config_file_if_not_exists(config: &Config) -> Result<(), ConfigError> {
        if !Self::config_path().exists() {
            config.print_if_verbose(
                format!(
                    "Creating configuration directory at {}...",
                    Self::config_dir().display()
                )
                .as_str(),
            );
            fs::create_dir_all(Self::config_dir())
                .map_err(|e| ConfigError::CreateDir(e.to_string()))?;
            config.print_if_verbose(
                format!(
                    "Creating configuration file at {}...",
                    Self::config_path().display()
                )
                .as_str(),
            );
            fs::File::create(Self::config_path())
                .map_err(|e| ConfigError::CreateFile(e.to_string()))?
                .write_all(b"{\n  \"$schema\": \"./config.schema.json\"\n}\n")
                .map_err(|e| ConfigError::CreateFile(e.to_string()))?;
        }

        // Keep the editor schema beside the config file so autocomplete also
        // works offline.
        let schema = Config::json_schema();
        let schema_path = Self::schema_path();
        let schema_is_current =
            fs::read_to_string(&schema_path).is_ok_and(|existing| existing.trim_end() == schema);
        if !schema_is_current {
            config.print_if_verbose(
                format!(
                    "Writing configuration schema at {}...",
                    schema_path.display()
                )
                .as_str(),
            );
            fs::write(&schema_path, format!("{schema}\n"))
                .map_err(|e| ConfigError::CreateFile(e.to_string()))?;
        }

        Self::add_schema_reference(config)?;

        Ok(())
    }

    /// Points an existing config at the schema file without rewriting the rest
    /// of it. Invalid JSON is left untouched so the user can fix it themselves.
    fn add_schema_reference(config: &Config) -> Result<(), ConfigError> {
        let config_path = Self::config_path();
        let Ok(contents) = fs::read_to_string(&config_path) else {
            return Ok(());
        };
        let Some(updated) = insert_schema_reference(&contents) else {
            return Ok(());
        };

        config.print_if_verbose(
            format!("Adding $schema reference to {}...", config_path.display()).as_str(),
        );
        fs::write(&config_path, updated).map_err(|e| ConfigError::CreateFile(e.to_string()))
    }

    fn read_config_file_if_exists(verbose: bool) -> Result<UserConfig, ConfigError> {
        let config_path = Config::config_path();
        if config_path.exists() {
            if verbose {
                println!("reading configuration file at {}", config_path.display());
            }
            let config = Self::read_config_file(&config_path, verbose)?;
            if verbose {
                println!("configuration loaded:");
                println!("{config}");
            }
            Ok(config)
        } else {
            if verbose {
                println!("no configuration file found, using defaults")
            }
            Ok(UserConfig::default())
        }
    }

    fn read_config_file(path: &Path, verbose: bool) -> Result<UserConfig, ConfigError> {
        // TODO: find way to partially process config and report err instead of current impl
        // (ignores all of user config)
        match fs::read_to_string(path) {
            Ok(data) => {
                Ok(serde_json::from_str(&data).map_err(|e| ConfigError::Parse(e.to_string())))?
            }
            Err(e) => {
                if verbose {
                    println!("error(s) in config file: {e:#?}");
                    println!("ignoring config...")
                }
                Ok(UserConfig::default())
            }
        }
    }
}

/// Inserts the `$schema` reference into a JSON config object while preserving
/// the rest of the file's contents and formatting. Returns `None` when the
/// config already references a schema, is not a JSON object, or cannot be
/// parsed.
fn insert_schema_reference(contents: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(contents).ok()?;
    if value.as_object()?.contains_key("$schema") {
        return None;
    }

    let open = contents.find('{')?;
    let closing = contents.rfind('}')?;
    let newline = if contents.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let reference = "\"$schema\": \"./config.schema.json\"";

    if contents[open + 1..closing].trim().is_empty() {
        let trailing = &contents[closing + 1..];
        return Some(format!("{{{newline}  {reference}{newline}}}{trailing}"));
    }

    // Match the indentation of the first property instead of imposing one.
    let indent = contents[open + 1..closing]
        .split_once('\n')
        .map(|(_, line)| line.trim_end_matches(['\r', '\n']))
        .and_then(|line| {
            let whitespace = line.len() - line.trim_start_matches([' ', '\t']).len();
            (whitespace > 0).then(|| &line[..whitespace])
        })
        .unwrap_or("  ");
    let insertion = format!("{newline}{indent}{reference},");

    let mut updated = String::with_capacity(contents.len() + insertion.len());
    updated.push_str(&contents[..=open]);
    updated.push_str(&insertion);
    updated.push_str(&contents[open + 1..]);
    Some(updated)
}

#[derive(Debug, PartialEq)]
pub enum ConfigError {
    CreateDir(String),
    CreateFile(String),
    Parse(String),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::CreateDir(err_msg) => {
                write!(f, "Unable to create configuration directory: {err_msg}")
            }
            ConfigError::CreateFile(err_msg) => {
                write!(f, "Unable to create configuration file: {err_msg}")
            }
            ConfigError::Parse(err_msg) => {
                write!(f, "Unable to parse configuration file: {err_msg}")
            }
        }
    }
}

impl Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::super::super::cli::Generator;
    use super::*;
    use crate::cli::SolidMode;
    use std::path::PathBuf;

    #[test]
    fn test_read_config_file_returns_default_for_missing_file() {
        let path = PathBuf::from("nonexistent_config.json");
        let config = Config::read_config_file(&path, false);
        assert_eq!(config, Ok(UserConfig::default()));
    }

    #[test]
    fn test_read_config_file_parses_correct_partial_config_when_frequency_defined() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(&path, r#"{ "frequency": "1w" }"#).unwrap();

        let config = Config::read_config_file(&path, false).expect("file should exist");
        assert_eq!(config.frequency, Some(Frequency::new("1w").unwrap()));
        assert_eq!(config.generators, None);
    }

    #[test]
    fn test_read_config_file_parses_correct_partial_config_when_generators_defined() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(
            &path,
            r#"{ "generators": ["spotlight", "julia", "nasa_apod", "solid"] }"#,
        )
        .unwrap();

        let config = Config::read_config_file(&path, false).expect("file should exist");
        assert_eq!(config.frequency, None);
        assert_eq!(
            config.generators,
            Some(Generators(Vec::from([
                Generator::Spotlight,
                Generator::Julia,
                Generator::NasaAPOD {
                    date: None,
                    random: false
                },
                Generator::Solid {
                    mode: SolidMode::Random
                }
            ])))
        );
    }

    #[test]
    fn test_read_config_file_parses_correct_empty_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(&path, r#"{}"#).unwrap();

        let config = Config::read_config_file(&path, false).expect("file should exist");
        assert_eq!(config, UserConfig::default());
    }

    #[test]
    fn test_read_config_file_ignores_schema_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(&path, r#"{ "$schema": "./config.schema.json" }"#).unwrap();

        let config = Config::read_config_file(&path, false).expect("file should exist");
        assert_eq!(config, UserConfig::default());
    }

    #[test]
    fn test_config_json_schema_is_draft07_with_expected_properties() {
        let schema: serde_json::Value =
            serde_json::from_str(&Config::json_schema()).expect("schema should be valid JSON");

        assert_eq!(schema["$schema"], "http://json-schema.org/draft-07/schema#");
        for property in [
            "auto_clean",
            "frequency",
            "generators",
            "julia_gen",
            "nasa_apod_gen",
            "solid_gen",
            "spotlight_gen",
            "themes",
        ] {
            assert!(
                schema["properties"][property].is_object(),
                "schema should describe `{property}`"
            );
        }
    }

    #[test]
    fn test_committed_config_schema_is_up_to_date() {
        let committed = include_str!("../../docs/config.schema.json").replace("\r\n", "\n");
        let generated = format!("{}\n", Config::json_schema());
        assert_eq!(
            committed, generated,
            "docs/config.schema.json is stale; regenerate it with `cargo run -- config --schema > docs/config.schema.json`"
        );
    }

    #[test]
    fn test_insert_schema_reference_preserves_existing_contents_and_indentation() {
        let input = "{\n\t\"frequency\": \"1d\",\n\t\"generators\": [\"spotlight\"]\n}\n";
        let expected = "{\n\t\"$schema\": \"./config.schema.json\",\n\t\"frequency\": \"1d\",\n\t\"generators\": [\"spotlight\"]\n}\n";
        assert_eq!(insert_schema_reference(input).as_deref(), Some(expected));
    }

    #[test]
    fn test_insert_schema_reference_handles_crlf() {
        let input = "{\r\n\t\"frequency\": \"1d\"\r\n}\r\n";
        let expected =
            "{\r\n\t\"$schema\": \"./config.schema.json\",\r\n\t\"frequency\": \"1d\"\r\n}\r\n";
        assert_eq!(insert_schema_reference(input).as_deref(), Some(expected));
    }

    #[test]
    fn test_insert_schema_reference_formats_empty_object() {
        assert_eq!(
            insert_schema_reference("{}\n").as_deref(),
            Some("{\n  \"$schema\": \"./config.schema.json\"\n}\n")
        );
    }

    #[test]
    fn test_insert_schema_reference_skips_existing_reference() {
        let input = r#"{ "$schema": "./config.schema.json", "frequency": "1d" }"#;
        assert_eq!(insert_schema_reference(input), None);
    }

    #[test]
    fn test_insert_schema_reference_skips_invalid_or_non_object_config() {
        assert_eq!(insert_schema_reference("{ not json"), None);
        assert_eq!(insert_schema_reference("[]"), None);
    }
}
