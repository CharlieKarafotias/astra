use super::super::super::wallpaper_generators::ApodDate;
use serde::Deserialize;
use std::fmt::{Display, Formatter, Write};

#[derive(Debug, Default, Deserialize, PartialEq)]
pub struct NasaApodConfig {
    date_from: Option<ApodDate>,
    date_to: Option<ApodDate>,
}

impl NasaApodConfig {
    pub fn date_from(&self) -> Option<String> {
        if let Some(d) = &self.date_from {
            Some(d.to_string())
        } else {
            None
        }
    }
    pub fn date_to(&self) -> Option<String> {
        if let Some(d) = &self.date_to {
            Some(d.to_string())
        } else {
            None
        }
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
