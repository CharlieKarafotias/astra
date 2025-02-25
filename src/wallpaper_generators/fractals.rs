use crate::wallpaper_generators::{create_color_map, Operator};
use super::{save_image, scale_image, WallpaperGeneratorError};
use image::ImageBuffer;
use num_complex::Complex;
use rand::random_range;
use std::path::PathBuf;
use log::debug;

// TODO: optimize algorithm using multithreading
pub fn generate_julia_set(width: u32, height: u32) -> Result<PathBuf, WallpaperGeneratorError> {
    let color_map = create_color_map(Operator::Gradient, 256, vec![
        [234, 224, 255],
        [221, 187, 255],
        [185, 134, 255],
        [128, 0, 255],
        [75, 0, 130],
    ]);
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
    debug!("Selected julia set: c={:?}", selected_julia_set);

    // Find hotspots and randomly select one
    let points_weights = sample_julia_set(selected_julia_set, width, height);
    let complex_hotspot = points_weights[random_range(0..points_weights.len())].0;
    debug!("Selected hotspot: z={:?}", complex_hotspot);

    let focus_pt = (complex_hotspot.re, complex_hotspot.im);
    let (scale_x, scale_y, start_x, start_y) =
        scale_image(3.0, 3.5, focus_pt, random_range(1.0..10.0));
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

            let pixel = imgbuf.get_pixel_mut(x, y);

            *pixel = image::Rgb(color_map[i]);
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
    let aspect_ratio = (width as f64 / height as f64).round() as u32;
    while points_weights.is_empty() && i < 5 {
        // Algorithm
        let height_ratio: u32 = 10 * (i + 1); // Backoff if fail to find points
        let width_ratio = aspect_ratio * height_ratio;
        let x_interval = width / width_ratio;
        let y_interval = height / height_ratio;
        let scaled_x = 3.0 / width as f64;
        let scaled_y = 3.5 / height as f64;

        for i in 0..width_ratio {
            for j in 0..height_ratio {
                let x = x_interval * i + (x_interval / 2);
                let y = y_interval * j + (y_interval / 2);
                let cx = x as f64 * scaled_x;
                let cy = y as f64 * scaled_y;
                let mut z = Complex::new(cx, cy);
                let mut i = 0;
                while i < 255 && z.norm() <= 2.0 {
                    z = z * z + c;
                    i += 1;
                }

                // TODO: make this customizable
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
