use super::super::themes::{ColorTheme, ThemeSelector};
use rand::seq::IndexedRandom;
use serde::Deserialize;
use std::fmt::{Display, Formatter, Write};

#[derive(Debug, Deserialize, PartialEq)]
pub struct ThemeConfig {
    name: String,
    colors: Vec<(u8, u8, u8)>,
    dark_mode_colors: Option<Vec<(u8, u8, u8)>>,
}

impl ThemeConfig {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn colors(&self) -> &Vec<(u8, u8, u8)> {
        &self.colors
    }

    pub fn dark_mode_colors(&self) -> &Option<Vec<(u8, u8, u8)>> {
        &self.dark_mode_colors
    }

    pub fn to_theme_selector(&self) -> ThemeSelector {
        ThemeSelector::from_color_theme(ColorTheme::new(
            self.name.clone(),
            self.dark_mode_colors.is_some(),
            self.colors.iter().map(|(r, g, b)| [*r, *g, *b]).collect(),
            self.dark_mode_colors()
                .as_ref()
                .map(|colors| colors.iter().map(|(r, g, b)| [*r, *g, *b]).collect()),
        ))
    }
}

impl Display for ThemeConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // only write if defined, else return empty string
        let mut s = String::new();
        writeln!(&mut s, "  name: {}", self.name)?;
        writeln!(&mut s, "    color(s): {:?}", self.colors)?;
        if let Some(colors) = &self.dark_mode_colors {
            writeln!(
                &mut s,
                "    dark_mode_color(s): {:?}",
                self.dark_mode_colors
            )?;
        }
        if s.len() != 0 {
            writeln!(f, "")?;
        }
        write!(f, "{s}")
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ThemeConfigs(Vec<ThemeConfig>);

impl ThemeConfigs {
    pub fn random(&self) -> &ThemeConfig {
        self.0.choose(&mut rand::rng()).expect("Failed to choose random theme because ThemeConfigs was empty - this should never happen")
    }
}

impl Display for ThemeConfigs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        for theme in &self.0 {
            writeln!(&mut s, "{theme}")?;
        }

        if s.len() != 0 {
            writeln!(f, "")?;
        }
        // If empty then show empty array, else wraps s in array
        write!(f, "[{s}]")
    }
}
