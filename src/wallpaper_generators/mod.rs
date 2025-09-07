mod bing_spotlight;
mod color_themes;
mod julia;
mod solid_color;
mod utils;

pub use bing_spotlight::generate_bing_spotlight;
pub use julia::generate_julia_set;
pub use solid_color::{Color, generate_solid_color};
pub use utils::{AstraImage, WallpaperGeneratorError, delete_wallpapers, handle_generate_options};
