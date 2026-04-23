use serde::Deserialize;
use std::fmt::{Display, Formatter, Write};

#[derive(Debug, Default, Deserialize, PartialEq)]
pub struct NasaApodConfig {
    date: Option<String>,
}

// TODO: should be date_range so if you want to see only a specific year then you can

impl NasaApodConfig {
    pub fn date(&self) -> Option<String> {
        self.date.clone()
    }
}

impl Display for NasaApodConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        if let Some(val) = &self.date {
            writeln!(&mut s, "    date: {}", val)?;
        }
        if !s.is_empty() {
            writeln!(f)?;
            s.pop(); // remove last newline character
        }
        write!(f, "{}", s)
    }
}
