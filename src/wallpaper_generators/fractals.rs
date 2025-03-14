use super::utils::{Operator, WallpaperGeneratorError, create_color_map, save_image, scale_image};
use crate::wallpaper_generators::color_themes::ThemeSelector;
use image::ImageBuffer;
use log::{debug, info};
use num_complex::Complex;
use rand::{random_iter, random_range};
use rayon::iter::ParallelIterator;
use std::path::PathBuf;

pub fn generate_julia_set(
    width: u32,
    height: u32,
    dark_mode: bool,
) -> Result<PathBuf, WallpaperGeneratorError> {
    info!("Generating julia set...");
    let theme = ThemeSelector::random();
    let selected_theme = theme.selected();
    info!("Theme: {selected_theme}");
    let color_map = create_color_map(
        Operator::Gradient,
        256,
        selected_theme.get_colors(dark_mode),
    );
    // Setup
    let julia_sets = vec![
        Complex::new(-0.79, 0.15),
        // TODO: currently broken due to poor sample julia set function, re-add once fixed
        // Complex::new(-0.162, 1.04),
        Complex::new(0.28, 0.008),
        // TODO: currently broken due to poor sample julia set function, re-add once fixed
        // Complex::new(0.3, -0.01),
        Complex::new(-1.476, 0.0),
        Complex::new(-0.12, -0.77),
        Complex::new(-0.70176, -0.3842),
        Complex::new(-0.4, 0.6),
        Complex::new(0.285, 0.01),
        // TODO: currently broken due to poor sample julia set function, re-add once fixed
        // Complex::new(0.0, 0.8),
        Complex::new(-0.835, 0.2321),
        Complex::new(-0.7269, 0.1889),
        // TODO: currently broken due to poor sample julia set function, re-add once fixed
        // Complex::new(0.4, 0.4),
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
    imgbuf
        .par_enumerate_pixels_mut()
        .for_each(|(x, y, mut pixel)| {
            let cx = x as f64 * (scale_x / width as f64) + start_x;
            let cy = y as f64 * (scale_y / height as f64) + start_y;

            let c = selected_julia_set;
            let mut z = Complex::new(cx, cy);

            let mut i = 0;
            while i < 255 && z.norm() <= 2.0 {
                z = z * z + c;
                i += 1;
            }
            *pixel = image::Rgb(color_map[i]);
        });

    let path_to_saved_image = save_image(&imgbuf)?;
    Ok(path_to_saved_image)
}

pub fn generate_mandelbrot_set(width: u32, height: u32) -> () {
    todo!("Implement for generating mandelbrot set")
}

// TODO: optimize algorithm using multithreading
// TODO: rethink this algorithm, I want it to be more random - aspect ratio will always be the same (what about aspect ratio and then random point between that)
// I'd like to have sections and then in between those sections, randomly select a point
// * * * *
// * * * *
// * * * *
// * * * *
// If sample results in dividing width and height by 2, then points (0,0) - (2,2) should be able to be selected, 1 at random
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
