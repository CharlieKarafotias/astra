mod os_implementations;
mod wallpaper_generators;

use os_implementations::{get_screen_resolution, update_wallpaper};
use wallpaper_generators::generate_julia_set;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (width, height) = get_screen_resolution()?;
    // println!("Screen resolution: {}x{}", width, height);
    // update_wallpaper("/Users/charlie/Desktop/test.jpg")?;
    generate_julia_set(width, height);
    Ok(())
}
