use super::super::cli::{Generator, SolidMode};
use super::{
    config::ConfigError,
    generators::{JuliaConfig, SolidConfig, SpotlightConfig},
};
use serde::Deserialize;
use std::cmp::PartialEq;
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
pub struct Frequency(String);

impl Frequency {
    pub fn new(s: &str) -> Result<Self, ConfigError> {
        Self::parse(s)
    }

    fn parse(s: &str) -> Result<Self, ConfigError> {
        let mut parsed = String::new();
        #[derive(Default, PartialEq)]
        enum Mode {
            #[default]
            Numeric,
            Unit,
            Done,
        }
        let mut mode = Mode::default();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            match mode {
                Mode::Numeric => {
                    if c.is_numeric() {
                        parsed.push(c);
                        if let Some(c) = chars.peek()
                            && !c.is_numeric()
                            && parsed.len() > 0
                        {
                            mode = Mode::Unit;
                        }
                    }
                }
                Mode::Unit => {
                    if ['s', 'm', 'h', 'd', 'w', 'M', 'y'].contains(&c) {
                        parsed.push(c);
                        mode = Mode::Done;
                    } else {
                        return Err(ConfigError::ParseError(
                            "unrecognized frequency unit, supported units are: seconds(s), minutes(m), hours(h), days(d), weeks(w), months(M), years(y)".to_string(),
                        ));
                    }
                }
                Mode::Done => {
                    return Err(ConfigError::ParseError(
                        "frequency is improperly formatted - example of frequency: 1w".to_string(),
                    ));
                }
            }
        }

        if parsed.len() == 0 {
            return Err(ConfigError::ParseError(
                "frequency must start with a number".to_string(),
            ));
        }
        if mode != Mode::Done {
            return Err(ConfigError::ParseError(
                "frequency must end with unit - examples are: s, m, h, d, w, M, y".to_string(),
            ));
        }

        Ok(Frequency(parsed.to_string()))
    }

    pub fn to_seconds(&self) -> u64 {
        let num = self.0[..self.0.len() - 1]
            .parse::<u64>()
            .expect("frequency must start with a number");
        let unit = self
            .0
            .chars()
            .last()
            .expect("frequency must end with a unit");
        match unit {
            's' => num,
            'm' => num * 60,
            'h' => num * 60 * 60,
            'd' => num * 60 * 60 * 24,
            'w' => num * 60 * 60 * 24 * 7,
            'M' => num * 60 * 60 * 24 * 30,
            'y' => num * 60 * 60 * 24 * 365,
            _ => panic!("unrecognized frequency unit"),
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
        let frequency = Frequency::new(s.as_str());
        match frequency {
            Ok(f) => Ok(f),
            Err(e) => Err(serde::de::Error::custom(e)),
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_frequency_to_seconds_seconds_format() {
        let f = Frequency::new("1s").unwrap();
        assert_eq!(1, f.to_seconds());
    }

    #[test]
    fn test_frequency_to_seconds_minutes_format() {
        let f = Frequency::new("1m").unwrap();
        assert_eq!(60, f.to_seconds());
    }

    #[test]
    fn test_frequency_to_seconds_hours_format() {
        let f = Frequency::new("2h").unwrap();
        assert_eq!(7200, f.to_seconds());
    }

    #[test]
    fn test_frequency_to_seconds_days_format() {
        let f = Frequency::new("1d").unwrap();
        assert_eq!(86400, f.to_seconds());
    }

    #[test]
    fn test_frequency_to_seconds_weeks_format() {
        let f = Frequency::new("1w").unwrap();
        assert_eq!(604800, f.to_seconds());
    }

    #[test]
    fn test_frequency_to_seconds_months_format() {
        let f = Frequency::new("1M").unwrap();
        assert_eq!(2592000, f.to_seconds());
    }

    #[test]
    fn test_frequency_to_seconds_years_format() {
        let f = Frequency::new("1y").unwrap();
        assert_eq!(31536000, f.to_seconds());
    }

    #[test]
    fn test_frequency_parse_unknown_unit_format() {
        let f = Frequency::new("1K");
        assert_eq!(Err(ConfigError::ParseError("unrecognized frequency unit, supported units are: seconds(s), minutes(m), hours(h), days(d), weeks(w), months(M), years(y)".to_string())), f);
    }

    #[test]
    fn test_frequency_parse_no_time_format() {
        let f = Frequency::new("d");
        assert_eq!(
            Err(ConfigError::ParseError(
                "frequency must start with a number".to_string()
            )),
            f
        );
    }

    #[test]
    fn test_frequency_parse_no_unit_format() {
        let f = Frequency::new("100");
        assert_eq!(
            Err(ConfigError::ParseError(
                "frequency must end with unit - examples are: s, m, h, d, w, M, y".to_string()
            )),
            f
        );
    }

    #[test]
    fn test_frequency_parse_no_empty_string() {
        let f = Frequency::new("");
        assert_eq!(
            Err(ConfigError::ParseError(
                "frequency must start with a number".to_string()
            )),
            f
        );
    }
}
