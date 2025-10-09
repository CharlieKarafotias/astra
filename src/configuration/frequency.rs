use super::config::ConfigError;
use serde::Deserialize;
use std::fmt::{Display, Formatter};

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
                            && !parsed.is_empty()
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
                        return Err(ConfigError::Parse(
                            "unrecognized frequency unit, supported units are: seconds(s), minutes(m), hours(h), days(d), weeks(w), months(M), years(y)".to_string(),
                        ));
                    }
                }
                Mode::Done => {
                    return Err(ConfigError::Parse(
                        "frequency is improperly formatted - example of frequency: 1w".to_string(),
                    ));
                }
            }
        }

        if parsed.is_empty() {
            return Err(ConfigError::Parse(
                "frequency must start with a number".to_string(),
            ));
        }
        if mode != Mode::Done {
            return Err(ConfigError::Parse(
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
    #[allow(unused_imports)]
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
        assert_eq!(Err(ConfigError::Parse("unrecognized frequency unit, supported units are: seconds(s), minutes(m), hours(h), days(d), weeks(w), months(M), years(y)".to_string())), f);
    }

    #[test]
    fn test_frequency_parse_no_time_format() {
        let f = Frequency::new("d");
        assert_eq!(
            Err(ConfigError::Parse(
                "frequency must start with a number".to_string()
            )),
            f
        );
    }

    #[test]
    fn test_frequency_parse_no_unit_format() {
        let f = Frequency::new("100");
        assert_eq!(
            Err(ConfigError::Parse(
                "frequency must end with unit - examples are: s, m, h, d, w, M, y".to_string()
            )),
            f
        );
    }

    #[test]
    fn test_frequency_parse_no_empty_string() {
        let f = Frequency::new("");
        assert_eq!(
            Err(ConfigError::Parse(
                "frequency must start with a number".to_string()
            )),
            f
        );
    }
}
