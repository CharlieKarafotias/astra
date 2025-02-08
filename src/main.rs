mod os_implementations;

use os_implementations::get_screen_resolution;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (width, height) = get_screen_resolution()?;
    println!("Screen resolution: {}x{}", width, height);
    Ok(())
}
