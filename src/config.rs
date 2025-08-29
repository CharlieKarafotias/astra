use crate::wallpaper_generators::Color;

use super::{
    cli::{Generator, SolidMode},
    constants::{APPLICATION, ORGANIZATION, QUALIFIER},
};
use directories::ProjectDirs;
use regex::Regex;
use serde::Deserialize;
use serde_json;
use std::{
    error::Error,
    fmt::{Display, Formatter, Write},
    fs,
    io::Write as ioWrite,
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
                    frequency: user_config.frequency,
                    generators: user_config.generators,
                    solid_gen: user_config.solid_gen,
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

    pub fn solid_gen(&self) -> Option<&SolidConfig> {
        if let Some(user_config) = &self.user_config {
            user_config.solid_gen.as_ref()
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

// TODO v1.1.0 - there are other fields users can customize. Every image type can have a mode.
// Solid Color generator: preferred colors from defaults, Random color, list of custom colors in RGB
// Spotlight generator: Country and locale - default is US and en-US, would be cool to allow specification
// looks to be [ISO_3166-1_alpha-2](https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2#US), test to confirm
// julia generator: custom complex numbers, custom color range, sample size for hotspots, more than just gradient for color pattern?
// Color themes: would be nice to utilize themes across generators
// bing spotlight can fetch <= 4 images. grab a few and check for closest matching theme? (provide escape hatch in solid config)
// solid_color can generate in color range if themes defined (provide escape hatch in solid config)
#[derive(Debug, Default, Deserialize, PartialEq)]
struct UserConfig {
    frequency: Option<Frequency>,
    generators: Option<Generators>,
    solid_gen: Option<SolidConfig>,
}

impl Display for UserConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut fields = vec![];

        macro_rules! push_field {
            ($field:ident) => {
                if let Some(val) = &self.$field {
                    fields.push(format!("{}: {}", stringify!($field), val));
                }
            };
        }

        push_field!(frequency);
        push_field!(generators);
        push_field!(solid_gen);

        write!(f, "{}", fields.join(", "))
    }
}

#[derive(Debug, Default, Deserialize, PartialEq)]
pub struct SolidConfig {
    preferred_default_colors: Option<Vec<Color>>,
    preferred_rgb_colors: Option<Vec<(u8, u8, u8)>>,
    // If true, ignore above fields
    respect_color_themes: Option<bool>,
}

impl SolidConfig {
    pub fn preferred_default_colors(&self) -> Option<Vec<Color>> {
        self.preferred_default_colors.clone()
    }
    pub fn preferred_rgb_colors(&self) -> Option<Vec<(u8, u8, u8)>> {
        self.preferred_rgb_colors.clone()
    }
    pub fn respect_color_themes(&self) -> Option<bool> {
        self.respect_color_themes
    }
}

impl Display for SolidConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // only write if defined, else return empty string
        let mut s = String::new();
        if let Some(val) = &self.preferred_default_colors {
            writeln!(&mut s, "  {:?}", val);
        }
        if let Some(val) = &self.preferred_rgb_colors {
            writeln!(&mut s, "  {:?}", val);
        }
        if let Some(val) = &self.respect_color_themes {
            writeln!(&mut s, "  {}", val);
        }
        write!(f, "{s}")
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

impl Display for Generators {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|g| g.prefix().to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
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
            return Err(ConfigError::ParseError(
                "frequency cannot be empty".to_string(),
            ));
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

impl Display for Frequency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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
            Err(ConfigError::ParseError(
                "frequency cannot be empty".to_string()
            ))
        );
    }
}
