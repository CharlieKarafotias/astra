use crate::wallpaper_generators::Color;
use serde::Deserialize;
use std::fmt::{Display, Formatter, Write};

#[derive(Debug, Default, Deserialize, PartialEq)]
pub struct SolidConfig {
    preferred_default_colors: Option<Vec<Color>>,
    preferred_rgb_colors: Option<Vec<(u8, u8, u8)>>,
    // If true, ignore above fields
    // TODO v1.1.0: generate in color range if themes defined (provide escape hatch in solid config)
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
            writeln!(&mut s, "    preferred_default_colors: {:?}", val)?;
        }
        if let Some(val) = &self.preferred_rgb_colors {
            writeln!(&mut s, "    preferred_rgb_colors: {:?}", val)?;
        }
        if let Some(val) = &self.respect_color_themes {
            writeln!(&mut s, "    respect_color_themes: {}", val)?;
        }
        if !s.is_empty() {
            writeln!(f)?;
            s.pop(); // remove last newline character
        }
        write!(f, "{s}")
    }
}
