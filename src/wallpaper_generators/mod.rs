mod bing_spotlight;
mod color_themes;
mod fractals;
mod utils;

pub use bing_spotlight::generate_bing_spotlight;
pub use fractals::generate_julia_set;
pub use utils::{AstraImage, WallpaperGeneratorError, delete_wallpapers, save_image};
