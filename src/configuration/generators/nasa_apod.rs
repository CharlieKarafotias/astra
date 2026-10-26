use super::super::super::wallpaper_generators::ApodDate;
use schemars::JsonSchema;
use serde::Deserialize;
use std::fmt::{Display, Formatter, Write};

/// Options for the NASA Astronomy Picture of the Day generator.
#[derive(Debug, Default, Deserialize, JsonSchema, PartialEq)]
pub struct NasaApodConfig {
    /// Earliest date to pick from, in `YYMMDD` format (for example `"260423"`).
    date_from: Option<ApodDate>,
    /// Latest date to pick from, in `YYMMDD` format (for example `"260423"`).
    date_to: Option<ApodDate>,
}

impl NasaApodConfig {
    pub fn date_from(&self) -> Option<String> {
        self.date_from.as_ref().map(|d| d.to_string())
    }
    pub fn date_to(&self) -> Option<String> {
        self.date_to.as_ref().map(|d| d.to_string())
    }
}

impl Display for NasaApodConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        if let Some(val) = &self.date_from {
            writeln!(&mut s, "    date_from: {}", val)?;
        }
        if let Some(val) = &self.date_to {
            writeln!(&mut s, "    date_to: {}", val)?;
        }
        if !s.is_empty() {
            writeln!(f)?;
            s.pop(); // remove last newline character
        }
        write!(f, "{}", s)
    }
}
