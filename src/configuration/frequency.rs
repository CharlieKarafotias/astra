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

    /// A function used by Linux implementation to convert Frequency to [OnCalendar](https://man.archlinux.org/man/systemd.time.7#CALENDAR_EVENTS) format
    pub fn as_on_calendar_format(&self) -> String {
        match self.to_seconds() {
            s if s < 60 => {
                // Event N seconds
                // ensure min interval of 1 to avoid OnCalendar error
                format!("*-*-* *:*:0/{}", s.max(1))
            }
            s if s < 3600 => {
                // Every N minutes
                let minutes = s / 60;
                format!("*-*-* *:0/{}", minutes.max(1))
            }
            s if s < 86400 => {
                // Every N hours
                let hours = s / 3600;
                format!("*-*-* 0/{}:00:00", hours.max(1))
            }
            s if s < 604800 => {
                // Every N days (up to 7 days)
                let days = s / 86400;
                format!("*-*-*/{} 00:00:00", days.max(1))
            }
            s if s < 2592000 => {
                // Every N weeks
                let weeks = s / 604800;
                if weeks <= 1 {
                    "weekly".to_string()
                } else {
                    format!("*-*-*/{} 00:00:00", weeks * 7)
                }
            }
            s if s < 31536000 => {
                // Every N months
                let months = s / 2592000;
                if months <= 1 {
                    "monthly".to_string()
                } else {
                    // No direct month interval syntax, use days
                    format!("*-*-*/{} 00:00:00", months * 30)
                }
            }
            // NOTE: for linux, only support 1y. if user uses 2y for example will still change
            // every year
            _ => "yearly".to_string(),
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

    #[test]
    fn test_frequency_to_on_calendar_fmt_seconds() {
        let f = Frequency::new("10s").unwrap();
        assert_eq!("*-*-* *:*:0/10".to_string(), f.as_on_calendar_format())
    }
    #[test]
    fn test_frequency_to_on_calendar_fmt_minutes() {
        let f = Frequency::new("15m").unwrap();
        assert_eq!("*-*-* *:0/15".to_string(), f.as_on_calendar_format())
    }
    #[test]
    fn test_frequency_to_on_calendar_fmt_hours() {
        let f = Frequency::new("6h").unwrap();
        assert_eq!("*-*-* 0/6:00:00".to_string(), f.as_on_calendar_format())
    }
    #[test]
    fn test_frequency_to_on_calendar_fmt_days() {
        let f = Frequency::new("2d").unwrap();
        assert_eq!("*-*-*/2 00:00:00".to_string(), f.as_on_calendar_format())
    }
    #[test]
    fn test_frequency_to_on_calendar_fmt_weeks() {
        let f = Frequency::new("2w").unwrap();
        assert_eq!("*-*-*/14 00:00:00".to_string(), f.as_on_calendar_format())
    }
    #[test]
    fn test_frequency_to_on_calendar_fmt_one_month() {
        let f = Frequency::new("1M").unwrap();
        assert_eq!("monthly".to_string(), f.as_on_calendar_format())
    }
    #[test]
    fn test_frequency_to_on_calendar_fmt_months() {
        let f = Frequency::new("2M").unwrap();
        assert_eq!("*-*-*/60 00:00:00".to_string(), f.as_on_calendar_format())
    }
    #[test]
    fn test_frequency_to_on_calendar_fmt_one_year() {
        let f = Frequency::new("1y").unwrap();
        assert_eq!("yearly".to_string(), f.as_on_calendar_format())
    }
    #[test]
    fn test_frequency_to_on_calendar_fmt_if_greater_than_one_year_go_to_yearly() {
        let f = Frequency::new("2y").unwrap();
        assert_eq!("yearly".to_string(), f.as_on_calendar_format())
    }
}
