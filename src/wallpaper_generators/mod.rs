mod bing_spotlight;
mod color_themes;
mod fractals;
mod solid_color;
mod utils;

pub use bing_spotlight::generate_bing_spotlight;
pub use fractals::generate_julia_set;
pub use solid_color::{Color, generate_solid_color};
pub use utils::{AstraImage, WallpaperGeneratorError, delete_wallpapers, save_image};

use crate::{
    cli::{ImageType, Mode, SolidMode},
    config::Config,
};

pub const ALL_GENERATORS: [ImageType; 3] = [
    ImageType::Julia,
    ImageType::Solid {
        mode: SolidMode::Random,
    },
    ImageType::Spotlight,
];

// TODO: could this live in the generator struct instead?
pub fn map_image_type_to_default(
    val: &ImageType,
) -> impl Fn(&Config) -> Result<AstraImage, WallpaperGeneratorError> {
    fn with_mode(
        f: fn(&Config, Option<&Mode>) -> Result<AstraImage, WallpaperGeneratorError>,
        mode: Option<&Mode>,
    ) -> impl Fn(&Config) -> Result<AstraImage, WallpaperGeneratorError> {
        move |config| f(config, mode)
    }

    match val {
        ImageType::Julia => with_mode(generate_julia_set, None),
        // TODO: fix issue here
        ImageType::Solid { mode } => with_mode(generate_solid_color, Some(&Mode::Solid(mode.clone()))),
        ImageType::Spotlight => with_mode(generate_bing_spotlight, None),
    }
}
