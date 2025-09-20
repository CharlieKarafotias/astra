use serde::Deserialize;
use std::fmt::{Display, Formatter, Write};

// looks to be [ISO_3166-1_alpha-2](https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2#US), test to confirm
#[derive(Debug, Default, Deserialize, PartialEq)]
pub struct SpotlightConfig {
    country: Option<String>,
    locale: Option<String>,
    // TODO v1.1.0 - not sure if this is worth it, can pull <= 4 images at a time and check closest matching theme
    respect_color_themes: Option<bool>,
}

impl SpotlightConfig {
    pub fn country(&self) -> Option<String> {
        self.country.clone()
    }

    pub fn locale(&self) -> Option<String> {
        self.locale.clone()
    }

    pub fn respect_color_themes(&self) -> Option<bool> {
        self.respect_color_themes
    }
}

impl Display for SpotlightConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        if let Some(val) = &self.country {
            writeln!(&mut s, "    country: {}", val)?;
        }
        if let Some(val) = &self.locale {
            writeln!(&mut s, "    locale: {}", val)?;
        }
        if let Some(val) = &self.respect_color_themes {
            writeln!(&mut s, "    respect_color_themes: {}", val)?;
        }
        if s.len() != 0 {
            writeln!(f, "")?;
            s.pop(); // remove last newline character
        }
        write!(f, "{}", s)
    }
}
