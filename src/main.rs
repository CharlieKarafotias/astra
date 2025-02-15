mod os_implementations;
mod wallpaper_generators;

use log::info;
use env_logger::{self, Builder, Env};
use os_implementations::{get_screen_resolution, update_wallpaper};
use wallpaper_generators::generate_julia_set;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Builder::from_env(Env::default().default_filter_or("info")).init();
    // TODO: add logger
    // TODO: make into CLI
    let (width, height) = get_screen_resolution()?;
    let saved_image_path = generate_julia_set(width, height)?;
    info!("Image saved to: {}", saved_image_path.display());
    update_wallpaper(saved_image_path)?;
    Ok(())
}
