use super::{
    cli::{Generator, SolidMode},
    constants::{APPLICATION, ORGANIZATION, QUALIFIER},
};
use directories::ProjectDirs;
use regex::Regex;
use serde::Deserialize;
use serde_json;
use std::{error::Error, fmt::Display, fs, path::Path};

pub struct Config {
    // From CLI options
    verbose: bool,

    // User configurations
    frequency: Option<Frequency>,
    generators: Option<Generators>,
}

#[derive(Debug, Default, Deserialize, PartialEq)]
struct UserConfig {
    frequency: Option<Frequency>,
    generators: Option<Generators>,
}

impl Config {
    pub fn new(verbose: bool) -> Self {
        match Config::read_config_file_if_exists(verbose) {
            Ok(user_config) => Self {
                verbose,
                frequency: user_config.frequency,
                generators: user_config.generators,
            },
            Err(e) => {
                if verbose {
                    println!("WARN - ignoring configuration due to error(s): {e}");
                }
                Self {
                    verbose,
                    frequency: None,
                    generators: None,
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
        self.generators.as_ref()
    }

    pub fn frequency(&self) -> Option<&Frequency> {
        self.frequency.as_ref()
    }

    fn read_config_file_if_exists(verbose: bool) -> Result<UserConfig, ConfigError> {
        let path = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
            .map(|dirs| dirs.data_dir().join("config.json"))
            .filter(|path| path.exists());
        match path {
            Some(existing_path) => {
                if verbose {
                    println!("reading configuration file at {}", existing_path.display());
                }
                Self::read_config_file(&existing_path)
            }
            None => {
                if verbose {
                    println!("no configuration file found, using defaults")
                }
                Ok(UserConfig::default())
            }
        }
    }

    fn read_config_file(path: &Path) -> Result<UserConfig, ConfigError> {
        match fs::read_to_string(path) {
            Ok(data) => {
                Ok(serde_json::from_str(&data).map_err(|e| ConfigError::ParseError(e.to_string())))?
            }
            Err(_) => Ok(UserConfig::default()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Generators(pub Vec<Generator>);

impl Generators {
    pub const ALL_GENERATORS: [Generator; 3] = [
        Generator::Julia,
        Generator::Solid {
            mode: SolidMode::Random,
        },
        Generator::Spotlight,
    ];
}

impl std::ops::Deref for Generators {
    type Target = Vec<Generator>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Generators {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
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
                Ok(Generators(parsed))
            }
            None => Ok(Generators(Vec::new())),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Frequency(String);

impl Frequency {
    fn to_seconds(self) -> u32 {
        todo!(
            "Implement to seconds when adding the automatic wallpaper adjuster feature for each OS"
        );
    }
}

impl<'de> Deserialize<'de> for Frequency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let re = Regex::new(r"^\d+(s|m|d|w|M|y)$").unwrap();
        if re.is_match(&s) {
            Ok(Frequency(s))
        } else {
            Err(serde::de::Error::custom(format!(
                "Invalid frequency format: {}",
                s
            )))
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ConfigError {
    ParseError(String),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::ParseError(err_msg) => {
                write!(f, "Unable to parse configuration file: {err_msg}")
            }
        }
    }
}

impl Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::SolidMode;
    use std::path::PathBuf;

    #[test]
    fn test_read_config_file_returns_default_for_missing_file() {
        let path = PathBuf::from("nonexistent_config.json");
        let config = Config::read_config_file(&path);
        assert_eq!(config, Ok(UserConfig::default()));
    }

    #[test]
    fn test_read_config_file_parses_correct_partial_config_when_frequency_defined() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(&path, r#"{ "frequency": "1w" }"#).unwrap();

        let config = Config::read_config_file(&path).expect("file should exist");
        assert_eq!(config.frequency, Some(Frequency("1w".to_string())));
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

        let config = Config::read_config_file(&path).expect("file should exist");
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

        let config = Config::read_config_file(&path).expect("file should exist");
        assert_eq!(config, UserConfig::default());
    }
}
