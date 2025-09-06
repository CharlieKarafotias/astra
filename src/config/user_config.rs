use super::super::cli::{Generator, SolidMode};
use super::{
    config::ConfigError,
    generators::{JuliaConfig, SolidConfig, SpotlightConfig},
};
use regex::Regex;
use serde::Deserialize;
use std::fmt::{Display, Formatter};

#[derive(Debug, Default, Deserialize, PartialEq)]
pub(super) struct UserConfig {
    pub(super) auto_clean: Option<Frequency>,
    pub(super) frequency: Option<Frequency>,
    pub(super) generators: Option<Generators>,
    pub(super) julia_gen: Option<JuliaConfig>,
    pub(super) solid_gen: Option<SolidConfig>,
    pub(super) spotlight_gen: Option<SpotlightConfig>,
    // IF New user config fields, ensure you push_field! in Display impl below & update readme
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

        push_field!(auto_clean);
        push_field!(frequency);
        push_field!(generators);
        push_field!(julia_gen);
        push_field!(solid_gen);
        push_field!(spotlight_gen);

        write!(f, "{}", fields.join(", "))
    }
}

#[derive(Debug, PartialEq)]
pub struct Generators(pub(super) Vec<Generator>);

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
pub struct Frequency(pub String);

impl Frequency {
    pub fn to_seconds(&self) -> Result<u64, ConfigError> {
        if self.0.is_empty() {
            return Err(ConfigError::ParseError(
                "frequency cannot be empty".to_string(),
            ));
        }
        let num = self.0[..self.0.len() - 1].parse::<u64>().map_err(|_| {
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
        let re = Regex::new(r"^\d+([smhdwMy])$").unwrap();
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

mod tests {
    use super::*;

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
