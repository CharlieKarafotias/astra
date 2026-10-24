mod bing_spotlight;
mod julia;
mod nasa_apod;
mod solid_color;
mod utils;

pub use bing_spotlight::generate_bing_spotlight;
pub use julia::generate_julia_set;
pub use nasa_apod::{ApodDate, generate_nasa_apod, parse_yymmdd};
pub use solid_color::{Color, generate_solid_color};
pub use utils::{
    AstraImage, WallpaperGeneratorError, average_color, delete_wallpapers, handle_generate_options,
};
// Only the macOS implementation prunes the wallpaper manifest.
#[cfg(target_os = "macos")]
pub use utils::prune_wallpaper_manifest_file;
