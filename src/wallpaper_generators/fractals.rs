use super::{save_image, scale_image, WallpaperGeneratorError};
use image::ImageBuffer;
use num_complex::Complex;
use std::path::PathBuf;

// TODO: add more julia sets and implement Hotspot algorithm that zooms into interesting areas
// use wiki for julia sets: https://en.wikipedia.org/wiki/Julia_set
pub(crate) fn generate_julia_set(
    width: u32,
    height: u32,
) -> Result<PathBuf, WallpaperGeneratorError> {
    let focus_pt = (0.0, 0.25);
    let (scale_x, scale_y, start_x, start_y) = scale_image(3.0, 3.5, focus_pt, 1.5);
    let mut imgbuf = ImageBuffer::new(width, height);

    // Generate julia set
    for x in 0..width {
        for y in 0..height {
            let cx = x as f64 * (scale_x / width as f64) + start_x;
            let cy = y as f64 * (scale_y / height as f64) + start_y;

            let c = Complex::new(-0.70176, -0.3842);
            let mut z = Complex::new(cx, cy);

            let mut i = 0;
            while i < 255 && z.norm() <= 2.0 {
                z = z * z + c;
                i += 1;
            }

            let pixel = imgbuf.get_pixel_mut(x, y);
            let image::Rgb(data) = *pixel;
            *pixel = image::Rgb([data[0], data[1], i as u8]);
        }
    }

    let path_to_saved_image = save_image(&imgbuf)?;
    Ok(path_to_saved_image)
}

pub(crate) fn generate_mandelbrot_set(width: u32, height: u32) -> () {
    todo!("Implement for generating mandelbrot set")
}
