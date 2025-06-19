use crate::os_implementations::path_to_desktop_folder;
use image::{ImageBuffer, Rgb};
use log::{debug, info};
use std::{
    error::Error,
    fs::create_dir_all,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

/// Creates a folder named "astra_wallpapers" on the desktop.
///
/// # Returns
///
/// A `Result` containing the path to the created folder on success, or a
/// `WallpaperGeneratorError` on failure.
pub(super) fn create_wallpaper_folder() -> Result<PathBuf, WallpaperGeneratorError> {
    info!("Preparing astra_wallpaper folder on desktop...");
    let path = path_to_desktop_folder()
        .map_err(|e| WallpaperGeneratorError::OSError(e.to_string()))?
        .join("astra_wallpapers");
    create_dir_all(&path).map_err(|e| WallpaperGeneratorError::OSError(e.to_string()))?;
    Ok(path)
}

/// Enum that specifies the color map generation algorithm
pub(super) enum Operator {
    Gradient,
}

/// Generates a color map based on the given parameters.
///
/// # Arguments
///
/// * `op` - The color map generation algorithm to use.
/// * `steps` - The number of color map entries to generate.
/// * `colors` - The colors to base the color map on.
///
/// # Returns
///
/// A vector of color map entries.
pub(super) fn create_color_map(op: Operator, steps: usize, colors: &Vec<[u8; 3]>) -> Vec<[u8; 3]> {
    let mut color_map = Vec::with_capacity(steps);
    match op {
        Operator::Gradient => {
            if colors.len() == 1 {
                for _ in 0..steps {
                    color_map.push(colors[0]);
                }
            } else {
                let color_steps = (steps - 1) / (colors.len() - 1);
                for i in 0..steps {
                    let color_idx = (i as f64 / color_steps as f64).floor() as usize;
                    if color_idx == (colors.len() - 1) {
                        color_map.push(colors[color_idx]);
                    } else {
                        let new_color = mix_color(
                            colors[color_idx],
                            colors[color_idx + 1],
                            (i % color_steps) as f64 / color_steps as f64,
                        );
                        color_map.push(new_color);
                    }
                }
            }
        }
    }
    color_map
}

/// Interpolates between two colors to create a new color.
///
/// # Arguments
///
/// * `color1` - The first color.
/// * `color2` - The second color.
/// * `weight_color_2` - A value between 0 and 1 that determines how much of `color2` to include in the output.
///
/// # Returns
///
/// A new color that is a mix of `color1` and `color2`.
fn mix_color(color1: [u8; 3], color2: [u8; 3], weight_color_2: f64) -> [u8; 3] {
    let r = color1[0] as f64 * (1.0 - weight_color_2) + color2[0] as f64 * weight_color_2;
    let g = color1[1] as f64 * (1.0 - weight_color_2) + color2[1] as f64 * weight_color_2;
    let b = color1[2] as f64 * (1.0 - weight_color_2) + color2[2] as f64 * weight_color_2;
    [r as u8, g as u8, b as u8]
}

/// Saves the given image to a file in the desktop wallpaper folder.
///
/// The file is named using the current UNIX timestamp to ensure uniqueness.
/// The image is saved in PNG format.
///
/// # Arguments
///
/// * `prefix` - A string to prepend to the file name.
/// * `image` - A reference to the `ImageBuffer` containing the image to save.
///
/// # Returns
///
/// A `Result` containing the path to the saved image on success, or a
/// `WallpaperGeneratorError` on failure.
pub(super) fn save_image(
    prefix: &str,
    image: &ImageBuffer<Rgb<u8>, Vec<u8>>,
) -> Result<PathBuf, WallpaperGeneratorError> {
    let mut save_path = create_wallpaper_folder()?;

    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| WallpaperGeneratorError::OSError(e.to_string()))?;

    save_path = save_path.join(format!("{prefix}_{}.png", time.as_secs().to_string()));
    debug!("Saving image to: {}", save_path.display());
    image
        .save(&save_path)
        .map_err(|_| WallpaperGeneratorError::ImageSaveError)?;
    Ok(save_path)
}

/// Scales the range of the provided plane to generate a zoomed in image.
///
/// Given the original range of the plane, the center point of the
/// image to focus on, and a scale factor, this function returns the new
/// range of the plane and the start points of the image so that
/// the image will be centered and scaled accordingly while keeping
/// the focus point as the center of the image.
///
/// # Arguments
///
/// * `x_range` - The range of the original x-axis.
/// * `y_range` - The range of the original y-axis.
/// * `focus_pt` - The center point of the image to focus on.
/// * `scale_factor` - The scale factor to apply to the image.
///
/// # Returns
///
/// A tuple containing:
///   - the new range of the x-axis
///   - the new range of the y-axis
///   - the start point of the x-axis
///   -  the start point of the y-axis
pub(super) fn scale_image(
    x_range: f64,
    y_range: f64,
    focus_pt: (f64, f64),
    scale_factor: f64,
) -> (f64, f64, f64, f64) {
    // scale_factor 2 means halve the size of the image
    let scaled_x_range = x_range / scale_factor;
    let scaled_y_range = y_range / scale_factor;
    // get start points so if center of image then
    let x_start = focus_pt.0 - (scaled_x_range / 2.0);
    let y_start = focus_pt.1 - (scaled_y_range / 2.0);
    (scaled_x_range, scaled_y_range, x_start, y_start)
}
// --- Utils ---

// --- Errors ---
#[derive(Debug, PartialEq)]
pub(crate) enum WallpaperGeneratorError {
    ImageGenerationError(String),
    ImageSaveError,
    NetworkError(String),
    OSError(String),
    ParseError(String),
}

impl std::fmt::Display for WallpaperGeneratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WallpaperGeneratorError::ImageGenerationError(msg) => {
                write!(f, "Image Generation Error: {}", msg)
            }
            WallpaperGeneratorError::ImageSaveError => {
                write!(f, "Failed to save image to file")
            }
            WallpaperGeneratorError::NetworkError(msg) => {
                write!(f, "Network Error: {}", msg)
            }
            WallpaperGeneratorError::OSError(msg) => {
                write!(f, "OS Error: {}", msg)
            }
            WallpaperGeneratorError::ParseError(msg) => {
                write!(f, "Parse Error: {}", msg)
            }
        }
    }
}

impl Error for WallpaperGeneratorError {}
// --- Errors ---

// --- Tests ---
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_image() {
        let (x_range, y_range, x_start, y_start) = scale_image(10.0, 10.0, (0.0, 0.0), 2.0);
        assert_eq!(x_range, 5.0);
        assert_eq!(y_range, 5.0);
        assert_eq!(x_start, -2.5);
        assert_eq!(y_start, -2.5);
    }

    #[test]
    fn test_create_color_map_all_red() {
        let color_map = create_color_map(Operator::Gradient, 256, &vec![[255, 0, 0]]);
        assert_eq!(color_map.len(), 256);
        for color in color_map {
            assert_eq!(color, [255, 0, 0]);
        }
    }

    #[test]
    fn test_create_color_map_red_green() {
        let color_map = create_color_map(Operator::Gradient, 256, &vec![[255, 0, 0], [0, 255, 0]]);
        assert_eq!(color_map.len(), 256);
        assert_eq!(color_map[0], [255, 0, 0]);
        assert_eq!(color_map[255], [0, 255, 0]);
    }
}
// --- Tests ---
