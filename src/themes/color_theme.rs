use std::fmt::Display;

pub struct ColorTheme {
    name: String,
    supports_dark_mode: bool,
    colors: Vec<[u8; 3]>,
    colors_dark_mode: Option<Vec<[u8; 3]>>,
}

impl ColorTheme {
    pub fn new(
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

    /// Returns the average color of the theme.
    ///
    /// Reference https://stackoverflow.com/questions/649454/what-is-the-best-way-to-average-two-colors-that-define-a-linear-gradient
    /// Formula:
    ///
    /// `(r_avg, g_avg, b_avg) = (sqrt((R_0^2 + ... + R_n^2) / n), sqrt((G_0^2 + ... + G_n^2) / n), sqrt((B_0^2 + ... + B_n^2) / n))`
    ///
    /// # Arguments
    ///
    /// * `dark_mode` - Whether to use the dark mode colors.
    ///
    /// # Returns
    ///
    /// The average color of the theme.
    pub fn average_color(&self, dark_mode: bool) -> [u8; 3] {
        let colors = self.get_colors(dark_mode);
        let mut rgb_avg = (0.0, 0.0, 0.0);
        colors.iter().for_each(|color| {
            let r = color[0] as f64;
            let g = color[1] as f64;
            let b = color[2] as f64;
            rgb_avg.0 += r.powi(2);
            rgb_avg.1 += g.powi(2);
            rgb_avg.2 += b.powi(2);
        });
        [
            (rgb_avg.0 / colors.len() as f64).sqrt() as u8,
            (rgb_avg.1 / colors.len() as f64).sqrt() as u8,
            (rgb_avg.2 / colors.len() as f64).sqrt() as u8,
        ]
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
