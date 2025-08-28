use super::super::cli::SolidMode;
use super::super::config::Config;
use super::super::os_implementations::get_screen_resolution;
use super::utils::{AstraImage, WallpaperGeneratorError};
use clap::ValueEnum;
use image::{ImageBuffer, Rgb};
use serde::Deserialize;

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

pub fn generate_solid_color(
    config: &Config,
    mode: &SolidMode,
) -> Result<AstraImage, WallpaperGeneratorError> {
    config.print_if_verbose("Generating solid color...");

    let (width, height) =
        get_screen_resolution().map_err(|e| WallpaperGeneratorError::OSError(e.to_string()))?;
    config.print_if_verbose(format!("Detected screen resolution: {}x{}", width, height).as_str());

    config.print_if_verbose("Generating image...");

    if config.respect_user_config {
        // TODO: if respect color theme, use this and take priority. Figure this out when picture is more clear
        // if config.solid_gen().
        todo!("implement respect color theme here and make it priority")
        // TODO: otherwise logic is to check the preferred colors and custom colors and merge these to a list. Then random select and use
    }

    let imgbuf = match mode {
        SolidMode::Random => ImageBuffer::from_pixel(
            width,
            height,
            Rgb([
                rand::random::<u8>(),
                rand::random::<u8>(),
                rand::random::<u8>(),
            ]),
        ),
        SolidMode::Rgb { r, g, b } => {
            ImageBuffer::from_pixel(width, height, Rgb([r.clone(), g.clone(), b.clone()]))
        }
        SolidMode::Color { name } => {
            let (r, g, b) = name.rgb();
            ImageBuffer::from_pixel(width, height, Rgb([r, g, b]))
        }
    };

    config.print_if_verbose("Image generated!");

    Ok(imgbuf)
}
