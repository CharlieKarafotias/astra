use super::{
    color_theme::ColorTheme,
    default_themes::{
        ColorThemes, theme_aurora_glow, theme_candy_crush, theme_cyber_sunset, theme_fire_ice,
        theme_galaxy_voyage, theme_mystic_forest, theme_neon_dreams, theme_ocean_breeze,
        theme_retro_pop, theme_sunlit_meadow,
    },
};

pub struct ThemeSelector {
    selected: ColorTheme,
}

impl ThemeSelector {
    pub fn from_color_theme(theme: ColorTheme) -> ThemeSelector {
        ThemeSelector { selected: theme }
    }

    pub fn random() -> ThemeSelector {
        ThemeSelector::new(rand::random())
    }

    pub fn new(theme: ColorThemes) -> ThemeSelector {
        match theme {
            ColorThemes::AuroraGlow => ThemeSelector::from_color_theme(theme_aurora_glow()),
            ColorThemes::CandyCrush => ThemeSelector::from_color_theme(theme_candy_crush()),
            ColorThemes::CyberSunset => ThemeSelector::from_color_theme(theme_cyber_sunset()),
            ColorThemes::FireIce => ThemeSelector::from_color_theme(theme_fire_ice()),
            ColorThemes::GalaxyVoyage => ThemeSelector::from_color_theme(theme_galaxy_voyage()),
            ColorThemes::MysticForest => ThemeSelector::from_color_theme(theme_mystic_forest()),
            ColorThemes::NeonDreams => ThemeSelector::from_color_theme(theme_neon_dreams()),
            ColorThemes::OceanBreeze => ThemeSelector::from_color_theme(theme_ocean_breeze()),
            ColorThemes::RetroPop => ThemeSelector::from_color_theme(theme_retro_pop()),
            ColorThemes::SunlitMeadow => ThemeSelector::from_color_theme(theme_sunlit_meadow()),
        }
    }
    pub fn selected(&self) -> &ColorTheme {
        &self.selected
    }
}

impl Default for ThemeSelector {
    fn default() -> Self {
        ThemeSelector::random()
    }
}
