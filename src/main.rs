mod os_implementations;
mod wallpaper_generators;

use env_logger::{self, Builder, Env};
use log::info;
use os_implementations::{get_screen_resolution, is_dark_mode_active, update_wallpaper};
use wallpaper_generators::generate_julia_set;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Builder::from_env(Env::default().default_filter_or("info")).init();
    // TODO: add logger
    // TODO: make into CLI
    let (width, height) = get_screen_resolution()?;
    let dark_mode_active = is_dark_mode_active()?;
    let saved_image_path = generate_julia_set(width, height, dark_mode_active)?;
    info!("Image saved to: {}", saved_image_path.display());
    update_wallpaper(saved_image_path)?;
    Ok(())
}
