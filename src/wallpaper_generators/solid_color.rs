use super::super::{
    cli::SolidMode, configuration::Config, configuration::generators::julia::Appearance,
    os_implementations::get_screen_resolution, os_implementations::is_dark_mode_active,
    themes::ThemeSelector,
};
use super::utils::{AstraImage, WallpaperGeneratorError};
use clap::ValueEnum;
use image::{ImageBuffer, Rgb};
use rand::{Rng, rng};
use serde::Deserialize;

pub fn generate_solid_color(
    config: &Config,
    mode: &SolidMode,
) -> Result<AstraImage, WallpaperGeneratorError> {
    config.print_if_verbose("Generating solid color image...");

    let (width, height) =
        get_screen_resolution().map_err(|e| WallpaperGeneratorError::OS(e.to_string()))?;
    config.print_if_verbose(format!("Detected screen resolution: {}x{}", width, height).as_str());

    if config.respect_user_config {
        config.print_if_verbose("User config detected with solid_gen options...");

        // Current setup will always prefer user_theme to config setup, need to decide if this is desired behavior
        let should_respect_color_themes =
            crate::respect_user_config_or_default!(config, solid_gen, respect_color_themes, {
                Ok(false)
            })?;
        let theme = match (should_respect_color_themes, config.themes()) {
            (true, Some(themes)) => themes.random().to_theme_selector(),
            (true, None) | (false, _) => ThemeSelector::random(),
        };
        let selected_theme = theme.selected();
        let appearance: Appearance =
            crate::respect_user_config_or_default!(config, julia_gen, appearance, {
                Ok(Appearance::Auto)
            })?;
        let dark_mode: bool = match appearance {
            Appearance::Auto => {
                is_dark_mode_active().map_err(|e| WallpaperGeneratorError::OS(e.to_string()))?
            }
            Appearance::Light => false,
            Appearance::Dark => true,
        };
        config.print_if_verbose(format!("Selected theme: {selected_theme}",).as_str());
        let [r, g, b] = selected_theme
            .average_color(dark_mode)
            .map_err(|e| WallpaperGeneratorError::ImageGeneration(e.to_string()))?;
        let imgbuf = generate_image(&SolidMode::Rgb { r, g, b }, width, height);
        config.print_if_verbose("Image generated!");
        return Ok(imgbuf);
    }

    let mut mode_options: Vec<SolidMode> = vec![];
    let preferred_default_colors: Vec<Color> =
        crate::respect_user_config_or_default!(config, solid_gen, preferred_default_colors, {
            Ok(vec![])
        })?;
    let preferred_rgb_colors: Vec<(u8, u8, u8)> =
        crate::respect_user_config_or_default!(config, solid_gen, preferred_rgb_colors, {
            Ok(vec![])
        })?;

    preferred_default_colors
        .iter()
        .for_each(|color| mode_options.push(SolidMode::Color { name: *color }));
    preferred_rgb_colors.iter().for_each(|(r, g, b)| {
        mode_options.push(SolidMode::Rgb {
            r: *r,
            g: *g,
            b: *b,
        })
    });

    let imgbuf: AstraImage = if mode_options.is_empty() {
        config.print_if_verbose(
            "read preferred_default_colors & preferred_rgb_colors config, but none were found",
        );
        // Use mode passed in instead since no config setup
        generate_image(mode, width, height)
    } else {
        config.print_if_verbose(
            "selecting random mode based on preferred_default_colors & preferred_rgb_colors config",
        );
        let mut rng = rng();
        let n = rng.random_range(..mode_options.len());
        let rand_mode = mode_options
            .get(n)
            .expect("random selected solid mode from user config should be defined");
        generate_image(rand_mode, width, height)
    };

    config.print_if_verbose("Image generated!");
    Ok(imgbuf)
}

fn generate_image(mode: &SolidMode, width: u32, height: u32) -> AstraImage {
    match mode {
        SolidMode::Random => ImageBuffer::from_pixel(
            width,
            height,
            Rgb([
                rand::random::<u8>(),
                rand::random::<u8>(),
                rand::random::<u8>(),
            ]),
        ),
        SolidMode::Rgb { r, g, b } => ImageBuffer::from_pixel(width, height, Rgb([*r, *g, *b])),
        SolidMode::Color { name } => {
            let (r, g, b) = name.rgb();
            ImageBuffer::from_pixel(width, height, Rgb([r, g, b]))
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize, PartialEq, ValueEnum)]
pub enum Color {
    White,
    Black,
    LightGray,
    DarkGray,
    Silver,
    SlateGray,
    NavyBlue,
    SkyBlue,
    SteelBlue,
    Teal,
    ForestGreen,
    Olive,
    Lime,
    Maroon,
    Crimson,
    DeepPurple,
    Indigo,
    Orchid,
    Coral,
    Beige,
}

impl Color {
    fn rgb(&self) -> (u8, u8, u8) {
        match self {
            Color::White => (255, 255, 255),
            Color::Black => (0, 0, 0),
            Color::LightGray => (211, 211, 211),
            Color::DarkGray => (64, 64, 64),
            Color::Silver => (192, 192, 192),
            Color::SlateGray => (112, 128, 144),
            Color::NavyBlue => (0, 0, 128),
            Color::SkyBlue => (135, 206, 235),
            Color::SteelBlue => (70, 130, 180),
            Color::Teal => (0, 128, 128),
            Color::ForestGreen => (34, 139, 34),
            Color::Olive => (128, 128, 0),
            Color::Lime => (0, 255, 0),
            Color::Maroon => (128, 0, 0),
            Color::Crimson => (220, 20, 60),
            Color::DeepPurple => (75, 0, 130),
            Color::Indigo => (75, 0, 130),
            Color::Orchid => (218, 112, 214),
            Color::Coral => (255, 127, 80),
            Color::Beige => (245, 245, 220),
        }
    }
}
