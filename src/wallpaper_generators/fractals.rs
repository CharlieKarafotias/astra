use super::super::config::Config;
use super::super::os_implementations::{get_screen_resolution, is_dark_mode_active};
use super::utils::{AstraImage, Operator, WallpaperGeneratorError, create_color_map, scale_image};
use crate::wallpaper_generators::color_themes::ThemeSelector;
use image::{ImageBuffer, Rgb};
use num_complex::Complex;
use rand::random_range;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

pub fn generate_julia_set(config: &Config) -> Result<AstraImage, WallpaperGeneratorError> {
    config.print_if_verbose("Generating julia set...");
    let (width, height) =
        get_screen_resolution().map_err(|e| WallpaperGeneratorError::OSError(e.to_string()))?;
    config.print_if_verbose(format!("Detected screen resolution: {}x{}", width, height).as_str());

    let dark_mode =
        is_dark_mode_active().map_err(|e| WallpaperGeneratorError::OSError(e.to_string()))?;
    config.print_if_verbose(format!("Is dark mode active: {}", dark_mode).as_str());

    let theme = ThemeSelector::random();
    let selected_theme = theme.selected();
    config.print_if_verbose(format!("Selected theme: {selected_theme}",).as_str());

    let color_map = create_color_map(
        Operator::Gradient,
        256,
        selected_theme.get_colors(dark_mode),
    );
    // Setup
    let julia_sets = vec![
        Complex::new(-0.79, 0.15),
        Complex::new(0.28, 0.008),
        Complex::new(-1.476, 0.0),
        Complex::new(-0.12, -0.77),
        Complex::new(-0.70176, -0.3842),
        Complex::new(-0.4, 0.6),
        Complex::new(0.285, 0.01),
        Complex::new(-0.835, 0.2321),
        Complex::new(-0.7269, 0.1889),
        Complex::new(0.4, 0.4),
        Complex::new(-0.162, 1.04),
        Complex::new(0.3, -0.01),
        Complex::new(0.0, 0.8),
    ];
    let selected_julia_set = julia_sets[random_range(0..julia_sets.len())];
    config.print_if_verbose(format!("Selected julia set: {}", selected_julia_set).as_str());

    // Find hotspots and randomly select one
    let points_weights = sample_julia_set(selected_julia_set, width, height);
    let complex_hotspot = points_weights[random_range(0..points_weights.len())].0;
    config.print_if_verbose(format!("Selected hotspot: {}", complex_hotspot).as_str());

    let focus_pt = (complex_hotspot.re, complex_hotspot.im);
    let (scale_x, scale_y, start_x, start_y) =
        scale_image(3.0, 3.5, focus_pt, random_range(1.0..10.0));
    let mut imgbuf = ImageBuffer::new(width, height);
    config.print_if_verbose("Generating image...");

    // Generate full julia set
    imgbuf.par_enumerate_pixels_mut().for_each(|(x, y, pixel)| {
        let cx = x as f64 * (scale_x / width as f64) + start_x;
        let cy = y as f64 * (scale_y / height as f64) + start_y;

        let c = selected_julia_set;
        let mut z = Complex::new(cx, cy);

        let mut i = 0;
        while i < 255 && z.norm() <= 2.0 {
            z = z * z + c;
            i += 1;
        }
        *pixel = Rgb(color_map[i]);
    });

    config.print_if_verbose("Image generated!");

    Ok(imgbuf)
}

fn sample_julia_set(c: Complex<f64>, width: u32, height: u32) -> Vec<(Complex<f64>, u32)> {
    let mut points_weights = vec![];
    let mut backoff_count = 0;
    let backoff_max = 15;
    // Means 200 iterations or more required to become a hotspot
    let mut dynamic_threshold_for_point_to_be_selected = 200;
    let threshold_decrease = 200 / backoff_max;
    let segments = 10;
    let aspect_ratio = (width as f64 / height as f64).round() as u32;

    while points_weights.is_empty() && backoff_count < backoff_max {
        // Algorithm
        let num_height_segments: u32 = segments * (backoff_count + 1);
        let num_width_segments = aspect_ratio * num_height_segments;
        let x_interval = width / num_width_segments;
        let y_interval = height / num_height_segments;
        let scaled_x = 3.0 / width as f64;
        let scaled_y = 3.5 / height as f64;

        let points: Vec<(Complex<f64>, u32)> = (0..(num_width_segments * num_height_segments))
            .into_par_iter()
            .map(|iteration| {
                let x = x_interval * (iteration % num_width_segments)
                    + random_range(0..(x_interval / 2));
                let y = y_interval * (iteration / num_width_segments)
                    + random_range(0..(y_interval / 2));
                let cx = x as f64 * scaled_x;
                let cy = y as f64 * scaled_y;
                // debug!("ITERATION: {} - x: {}, y: {}, cx: {}, cy: {}", iteration, x, y, cx, cy);
                let mut z = Complex::new(cx, cy);
                let mut i = 0;
                while i < 255 && z.norm() <= 2.0 {
                    z = z * z + c;
                    i += 1;
                }

                if i > dynamic_threshold_for_point_to_be_selected {
                    Some((Complex::new(cx, cy), i))
                } else {
                    None
                }
            })
            .flatten()
            .collect();

        points_weights.extend(points);

        // Backoff increment
        backoff_count += 1;
        dynamic_threshold_for_point_to_be_selected -= threshold_decrease;
    }
    points_weights.sort_by(|(_, w1), (_, w2)| w2.cmp(w1));
    points_weights
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_sample_julia_set() {
        let points = super::sample_julia_set(super::Complex::new(0.4, 0.4), 800, 600);
        assert!(!points.is_empty());
    }
}
