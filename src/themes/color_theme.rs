use std::fmt::Display;

pub struct ColorTheme {
    name: String,
    supports_dark_mode: bool,
    colors: Vec<[u8; 3]>,
    colors_dark_mode: Option<Vec<[u8; 3]>>,
}

impl ColorTheme {
    pub(super) fn new(
        name: String,
        supports_dark_mode: bool,
        colors: Vec<[u8; 3]>,
        colors_dark_mode: Option<Vec<[u8; 3]>>,
    ) -> Self {
        Self {
            name,
            supports_dark_mode,
            colors,
            colors_dark_mode,
        }
    }

    pub fn get_colors(&self, dark_mode: bool) -> &Vec<[u8; 3]> {
        if dark_mode && self.supports_dark_mode {
            self.colors_dark_mode.as_ref().unwrap_or(&self.colors)
        } else {
            &self.colors
        }
    }
}

impl Display for ColorTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Color Theme: {}, supports dark mode: {}, color_count: {}",
            self.name,
            self.supports_dark_mode,
            self.colors.len()
        )
    }
}
