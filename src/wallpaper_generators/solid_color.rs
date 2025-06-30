use super::super::os_implementations::get_screen_resolution;
use super::utils::{AstraImage, WallpaperGeneratorError};
use image::{ImageBuffer, Rgb};

pub enum Color {
    Random,
    Solid { r: u8, g: u8, b: u8 },
    Named { name: &'static str },
}
pub fn generate_solid_color(
    color: Color,
    verbose: bool,
) -> Result<AstraImage, WallpaperGeneratorError> {
    if verbose {
        println!("Generating solid color...");
    }
    let (width, height) =
        get_screen_resolution().map_err(|e| WallpaperGeneratorError::OSError(e.to_string()))?;
    if verbose {
        println!("Detected screen resolution: {}x{}", width, height);
    }

    if verbose {
        println!("Generating image...");
    }

    let imgbuf = match color {
        Color::Random => ImageBuffer::from_pixel(
            width,
            height,
            Rgb([
                rand::random::<u8>(),
                rand::random::<u8>(),
                rand::random::<u8>(),
            ]),
        ),
        Color::Solid { r, g, b } => ImageBuffer::from_pixel(width, height, Rgb([r, g, b])),
        Color::Named { name } => {
            let color = COLORS
                .get(name)
                .ok_or(WallpaperGeneratorError::InvalidColorName(name.to_string()))?;

            ImageBuffer::from_pixel(width, height, Rgb([color.0, color.1, color.2]))
        }
    };

    if verbose {
        println!("Image generated!");
    }

    Ok(imgbuf)
}

use phf::phf_map;

static COLORS: phf::Map<&'static str, (u8, u8, u8)> = phf_map! {
    "White" => (255, 255, 255),
    "Black" => (0, 0, 0),
    "LightGray" => (211, 211, 211),
    "DarkGray" => (64, 64, 64),
    "Silver" => (192, 192, 192),
    "SlateGray" => (112, 128, 144),
    "NavyBlue" => (0, 0, 128),
    "SkyBlue" => (135, 206, 235),
    "SteelBlue" => (70, 130, 180),
    "Teal" => (0, 128, 128),
    "ForestGreen" => (34, 139, 34),
    "Olive" => (128, 128, 0),
    "Lime" => (0, 255, 0),
    "Maroon" => (128, 0, 0),
    "Crimson" => (220, 20, 60),
    "DeepPurple" => (75, 0, 130),
    "Indigo" => (75, 0, 130),
    "Orchid" => (218, 112, 214),
    "Coral" => (255, 127, 80),
    "Beige" => (245, 245, 220),
};

pub fn color_options_by_name() -> Vec<&'static str> {
    COLORS.keys().copied().collect()
}
