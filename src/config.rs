use super::{
    cli::ImageType,
    constants::{APPLICATION, ORGANIZATION, QUALIFIER},
};
use directories::ProjectDirs;
use serde::Deserialize;
use serde_json;
use std::{fs, path::Path};

pub struct Config {
    // From CLI options
    verbose: bool,

    // User configurations
    frequency: Option<String>,
    generators: Option<Vec<ImageType>>,
}

#[derive(Debug, Default, Deserialize, PartialEq)]
struct UserConfig {
    frequency: Option<String>,
    #[serde(default, deserialize_with = "deserialize_generators")]
    generators: Option<Vec<ImageType>>,
}

impl Config {
    pub fn new(verbose: bool) -> Self {
        let UserConfig {
            frequency,
            generators,
        } = Config::read_config_file_if_exists();
        Self {
            frequency,
            generators,
            verbose,
        }
    }

    pub fn print_if_verbose(&self, message: &str) {
        if self.verbose {
            println!("{}", message);
        }
    }

    fn read_config_file_if_exists() -> UserConfig {
        ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
            .map(|dirs| dirs.data_dir().join("config.json"))
            .filter(|path| path.exists())
            .map(|path| Self::read_config_file(&path))
            .unwrap_or_default()
    }

    fn read_config_file(path: &Path) -> UserConfig {
        match fs::read_to_string(path) {
            Ok(data) => serde_json::from_str(&data).unwrap_or_default(), // TODO: add proper error handling to inform user of config error instead of default
            Err(_) => UserConfig::default(),
        }
    }
}

fn deserialize_generators<'de, D>(deserializer: D) -> Result<Option<Vec<ImageType>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw: Option<Vec<String>> = Option::deserialize(deserializer)?;
    match raw {
        Some(list) => {
            let parsed = list
                .into_iter()
                .map(|s| s.parse().map_err(serde::de::Error::custom))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Some(parsed))
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::SolidMode;
    use std::path::PathBuf;

    #[test]
    fn test_read_config_file_returns_default_for_missing_file() {
        let path = PathBuf::from("nonexistent_config.json");
        let config = Config::read_config_file(&path);
        assert_eq!(config, UserConfig::default());
    }

    #[test]
    fn test_read_config_file_parses_correct_partial_config_when_frequency_defined() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(&path, r#"{ "frequency": "1w" }"#).unwrap();

        let config = Config::read_config_file(&path);
        assert_eq!(config.frequency, Some("1w".to_string()));
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

        let config = Config::read_config_file(&path);
        assert_eq!(config.frequency, None);
        assert_eq!(
            config.generators,
            Some(Vec::from([
                ImageType::Spotlight,
                ImageType::Julia,
                ImageType::Solid {
                    mode: SolidMode::Random
                }
            ]))
        );
    }

    #[test]
    fn test_read_config_file_parses_correct_empty_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(&path, r#"{}"#).unwrap();

        let config = Config::read_config_file(&path);
        assert_eq!(config, UserConfig::default());
    }
}
