use super::{save_image, scale_image, WallpaperGeneratorError};
use image::ImageBuffer;
use num_complex::Complex;
use rand::random_range;
use std::path::PathBuf;

// TODO: optimize algorithm using multithreading
pub fn generate_julia_set(width: u32, height: u32) -> Result<PathBuf, WallpaperGeneratorError> {
    // Setup
    let julia_sets = vec![
        Complex::new(-0.70176, -0.3842),
        Complex::new(-0.4, 0.6),
        Complex::new(0.285, 0.01),
        Complex::new(0.0, 0.8),
        Complex::new(-0.835, 0.2321),
        Complex::new(-0.7269, 0.1889),
        Complex::new(0.4, 0.4),
    ];
    let selected_julia_set = julia_sets[random_range(0..julia_sets.len())];
    println!("Selected julia set: c={:?}", selected_julia_set);

    // Find hotspots and randomly select one
    let points_weights = sample_julia_set(selected_julia_set, width, height);
    let complex_hotspot = points_weights[random_range(0..points_weights.len())].0;

    let focus_pt = (complex_hotspot.re, complex_hotspot.im);
    let (scale_x, scale_y, start_x, start_y) =
        scale_image(3.0, 3.5, focus_pt, random_range(2.0..5.0));
    let mut imgbuf = ImageBuffer::new(width, height);

    // Generate full julia set
    for x in 0..width {
        for y in 0..height {
            let cx = x as f64 * (scale_x / width as f64) + start_x;
            let cy = y as f64 * (scale_y / height as f64) + start_y;

            let c = selected_julia_set;
            let mut z = Complex::new(cx, cy);

            let mut i = 0;
            while i < 255 && z.norm() <= 2.0 {
                z = z * z + c;
                i += 1;
            }

            // TODO: randomize color
            let pixel = imgbuf.get_pixel_mut(x, y);
            let image::Rgb(data) = *pixel;
            *pixel = image::Rgb([data[0], data[1], i as u8]);
        }
    }

    let path_to_saved_image = save_image(&imgbuf)?;
    Ok(path_to_saved_image)
}

pub fn generate_mandelbrot_set(width: u32, height: u32) -> () {
    todo!("Implement for generating mandelbrot set")
}

// TODO: optimize algorithm using multithreading
fn sample_julia_set(c: Complex<f64>, width: u32, height: u32) -> Vec<(Complex<f64>, u32)> {
    let mut points_weights = vec![];
    let mut i = 0;
    while points_weights.is_empty() && i < 5 {
        // Algorithm
        let height_ratio: u32 = 10 * (i + 1); // Backoff if fail to find points
        let width_ratio = (width as f64 / height as f64).round() as u32 * height_ratio;
        let x_interval = width / width_ratio;
        let y_interval = height / height_ratio;
        for i in 0..width_ratio {
            for j in 0..height_ratio {
                let x = x_interval * i + (x_interval / 2);
                let y = y_interval * j + (y_interval / 2);
                let cx = x as f64 * (3.0 / width as f64);
                let cy = y as f64 * (3.5 / height as f64);
                let mut z = Complex::new(cx, cy);
                let mut i = 0;
                while i < 255 && z.norm() <= 2.0 {
                    z = z * z + c;
                    i += 1;
                }

                if i > 100 {
                    points_weights.push((Complex::new(cx, cy), i));
                }
            }
        }

        // Backoff increment
        i += 1;
    }
    points_weights.sort_by(|(_, w1), (_, w2)| w2.cmp(w1));
    points_weights
}
