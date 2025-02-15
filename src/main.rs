mod os_implementations;
mod wallpaper_generators;

use os_implementations::{get_screen_resolution, update_wallpaper};
use wallpaper_generators::generate_julia_set;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (width, height) = get_screen_resolution()?;
    let saved_image_path = generate_julia_set(width, height)?;
    println!("Saved image to: {}", saved_image_path.display());
    update_wallpaper(saved_image_path)?;
    Ok(())
}
