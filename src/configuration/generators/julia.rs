use schemars::JsonSchema;
use serde::Deserialize;
use std::fmt::{Display, Formatter, Write};

/// Options for the Julia fractal generator.
#[derive(Debug, Default, Deserialize, JsonSchema, PartialEq)]
pub struct JuliaConfig {
    /// Color appearance to use: `Auto` follows the system appearance.
    appearance: Option<Appearance>,
    /// Complex numbers as `[real, imaginary]` pairs. One is picked at random.
    complex_numbers: Option<Vec<(f64, f64)>>,
    // Iterations required to become a hotspot (higher = more detailed)
    /// Iterations needed before a point counts as a hotspot (0-255). Higher
    /// values generally add color variety.
    starting_sample_threshold: Option<u8>,
    /// Use the color themes from the top-level `themes` setting.
    respect_color_themes: Option<bool>,
}

// TODO: relocate to color_themes
/// Preferred color appearance.
#[derive(Copy, Clone, Debug, Deserialize, JsonSchema, PartialEq)]
pub(crate) enum Appearance {
    Auto,
    Light,
    Dark,
}

impl JuliaConfig {
    pub fn appearance(&self) -> Option<Appearance> {
        self.appearance
    }

    pub fn complex_numbers(&self) -> Option<Vec<(f64, f64)>> {
        self.complex_numbers.clone()
    }

    pub fn starting_sample_threshold(&self) -> Option<u8> {
        self.starting_sample_threshold
    }

    pub fn respect_color_themes(&self) -> Option<bool> {
        self.respect_color_themes
    }
}

impl Display for JuliaConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // only write if defined, else return empty string
        let mut s = String::new();
        if let Some(val) = &self.appearance {
            writeln!(&mut s, "    appearance: {:?}", val)?;
        }
        if let Some(val) = &self.complex_numbers {
            writeln!(&mut s, "    complex_numbers: {:?}", val)?;
        }
        if let Some(val) = &self.starting_sample_threshold {
            writeln!(&mut s, "    starting_sample_threshold: {:?}", val)?;
        }
        if let Some(val) = &self.respect_color_themes {
            writeln!(&mut s, "    respect_color_themes: {:?}", val)?;
        }
        if !s.is_empty() {
            writeln!(f)?;
            s.pop(); // remove last newline character
        }
        write!(f, "{s}")
    }
}
