use schemars::JsonSchema;
use serde::Deserialize;
use std::fmt::{Display, Formatter, Write};

/// Options for the Bing Spotlight generator.
// looks to be [ISO_3166-1_alpha-2](https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2#US), test to confirm
#[derive(Debug, Default, Deserialize, JsonSchema, PartialEq)]
pub struct SpotlightConfig {
    /// Country whose Spotlight feed to use, as an ISO 3166-1 alpha-2 code.
    country: Option<String>,
    /// Locale variant for the feed, as a language-country code such as `"en-GB"`.
    locale: Option<String>,
    /// Prefer images that match the top-level color themes.
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
        if !s.is_empty() {
            writeln!(f)?;
            s.pop(); // remove last newline character
        }
        write!(f, "{}", s)
    }
}
