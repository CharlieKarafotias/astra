use super::super::constants::{APPLICATION, ORGANIZATION, QUALIFIER};
use super::{
    frequency::Frequency,
    generators::{Generators, JuliaConfig, SolidConfig, SpotlightConfig},
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
                .write_all(b"{}")
                .map_err(|e| ConfigError::CreateFile(e.to_string()))?;
        }
        Ok(())
    }

    fn read_config_file_if_exists(verbose: bool) -> Result<UserConfig, ConfigError> {
        let config_path = Config::config_path();
        if config_path.exists() {
            if verbose {
                println!("reading configuration file at {}", &config_path.display());
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
        // TODO v1.1.0 - if part of config fails, see if you can partially read. Right now if part is wrong, it respects nothing and defaults to old behavior
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
            r#"{ "generators": ["spotlight", "julia", "solid"] }"#,
        )
        .unwrap();

        let config = Config::read_config_file(&path, false).expect("file should exist");
        assert_eq!(config.frequency, None);
        assert_eq!(
            config.generators,
            Some(Generators(Vec::from([
                Generator::Spotlight,
                Generator::Julia,
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
}
