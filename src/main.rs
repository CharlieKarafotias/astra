mod os_implementations;

use crate::os_implementations::update_wallpaper;
use os_implementations::get_screen_resolution;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (width, height) = get_screen_resolution()?;
    println!("Screen resolution: {}x{}", width, height);
    update_wallpaper("/Users/charlie/Desktop/test.jpg")?;
    Ok(())
}
