use super::{
    cli::{Generator, SolidMode},
    constants::{APPLICATION, ORGANIZATION, QUALIFIER},
};
use directories::ProjectDirs;
use regex::Regex;
use serde::Deserialize;
use serde_json;
use std::fmt::Formatter;
use std::ops::Add;
use std::{
    error::Error,
    fmt::Display,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

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

impl Display for UserConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut buf = String::new();
        if let Some(frequency) = &self.frequency {
            buf = buf.add(format!("frequency: {}, ", frequency.0).as_str());
        }
        if let Some(generators) = &self.generators {
            buf = buf.add(format!("generators: {:?}, ", generators.0).as_str());
        }
        if buf.ends_with(", ") {
            buf.truncate(buf.len() - 2);
        }
        write!(f, "{buf}")
    }
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
                .map_err(|e| ConfigError::CreateDirError(e.to_string()))?;
            config.print_if_verbose(
                format!(
                    "Creating configuration file at {}...",
                    Self::config_path().display()
                )
                .as_str(),
            );
            fs::File::create(Self::config_path())
                .map_err(|e| ConfigError::CreateFileError(e.to_string()))?
                .write_all(b"{}")
                .map_err(|e| ConfigError::CreateFileError(e.to_string()))?;
        }
        Ok(())
    }

    fn read_config_file_if_exists(verbose: bool) -> Result<UserConfig, ConfigError> {
        let config_path = Config::config_path();
        if config_path.exists() {
            if verbose {
                println!("reading configuration file at {}", &config_path.display());
            }
            let config = Self::read_config_file(&config_path)?;
            if verbose {
                println!("configuration loaded - {config}");
            }
            Ok(config)
        } else {
            if verbose {
                println!("no configuration file found, using defaults")
            }
            Ok(UserConfig::default())
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
    fn to_seconds(&self) -> Result<u32, ConfigError> {
        if self.0.is_empty() {
            return Err(ConfigError::ParseError("frequency cannot be empty".to_string()));
        }
        let num = self.0[..self.0.len() - 1].parse::<u32>().map_err(|_| {
            ConfigError::ParseError("expected frequency to start with number".to_string())
        })?;
        let unit = self
            .0
            .chars()
            .last()
            .and_then(|c| if c.is_numeric() { None } else { Some(c) })
            .ok_or(ConfigError::ParseError("expected frequency to end with unit denoting seconds(s), minutes(m), hours(h), days(d), weeks(w), months(M), years(y)".to_string()))?;

        match unit {
            's' => Ok(num),
            'm' => Ok(num * 60),
            'h' => Ok(num * 60 * 60),
            'd' => Ok(num * 60 * 60 * 24),
            'w' => Ok(num * 60 * 60 * 24 * 7),
            'M' => Ok(num * 60 * 60 * 24 * 30),
            'y' => Ok(num * 60 * 60 * 24 * 365),
            _ => Err(ConfigError::ParseError("unrecognized frequency unit, supported units are: seconds(s), minutes(m), hours(h), days(d), weeks(w), months(M), years(y)".to_string())),
        }
    }
}

impl<'de> Deserialize<'de> for Frequency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let re = Regex::new(r"^\d+([smdwMy])$").unwrap();
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
    CreateDirError(String),
    CreateFileError(String),
    ParseError(String),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::CreateDirError(err_msg) => {
                write!(f, "Unable to create configuration directory: {err_msg}")
            }
            ConfigError::CreateFileError(err_msg) => {
                write!(f, "Unable to create configuration file: {err_msg}")
            }
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

    #[test]
    fn test_frequency_to_seconds_seconds_format() {
        assert_eq!(Frequency("1s".to_string()).to_seconds(), Ok(1));
    }

    #[test]
    fn test_frequency_to_seconds_minutes_format() {
        assert_eq!(Frequency("1m".to_string()).to_seconds(), Ok(60));
    }

    #[test]
    fn test_frequency_to_seconds_hours_format() {
        assert_eq!(Frequency("2h".to_string()).to_seconds(), Ok(7200));
    }

    #[test]
    fn test_frequency_to_seconds_days_format() {
        assert_eq!(Frequency("1d".to_string()).to_seconds(), Ok(86400));
    }

    #[test]
    fn test_frequency_to_seconds_weeks_format() {
        assert_eq!(Frequency("1w".to_string()).to_seconds(), Ok(604800));
    }

    #[test]
    fn test_frequency_to_seconds_months_format() {
        assert_eq!(Frequency("1M".to_string()).to_seconds(), Ok(2592000));
    }

    #[test]
    fn test_frequency_to_seconds_years_format() {
        assert_eq!(Frequency("1y".to_string()).to_seconds(), Ok(31536000));
    }

    #[test]
    fn test_frequency_to_seconds_unknown_unit_format() {
        assert_eq!(
            Frequency("1K".to_string()).to_seconds(),
            Err(ConfigError::ParseError("unrecognized frequency unit, supported units are: seconds(s), minutes(m), hours(h), days(d), weeks(w), months(M), years(y)".to_string()))
        );
    }

    #[test]
    fn test_frequency_to_seconds_no_time_format() {
        assert_eq!(
            Frequency("d".to_string()).to_seconds(),
            Err(ConfigError::ParseError(
                "expected frequency to start with number".to_string()
            ))
        );
    }

    #[test]
    fn test_frequency_to_seconds_no_unit_format() {
        assert_eq!(
            Frequency("100".to_string()).to_seconds(),
            Err(ConfigError::ParseError("expected frequency to end with unit denoting seconds(s), minutes(m), hours(h), days(d), weeks(w), months(M), years(y)".to_string()))
        );
    }

    #[test]
    fn test_frequency_to_seconds_no_empty_string() {
        assert_eq!(
            Frequency("".to_string()).to_seconds(),
            Err(ConfigError::ParseError("frequency cannot be empty".to_string()))
        );
    }
}
