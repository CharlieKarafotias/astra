use super::super::{
    configuration::{Config, Frequency},
    constants::{APPLICATION, ORGANIZATION, QUALIFIER},
};
use crate::cli::Generator;
use crate::os_implementations::update_wallpaper;
use directories::ProjectDirs;
use image::{ImageBuffer, Rgb};
use std::{
    error::Error,
    fmt,
    fs::{create_dir_all, read_dir, remove_dir_all, remove_file},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

pub type AstraImage = ImageBuffer<Rgb<u8>, Vec<u8>>;

/// Creates a folder named "wallpapers" under the data_dir folder of Astra.
/// For each path, see: https://lib.rs/crates/directories
///
/// # Returns
///
/// A `Result` containing the path to the created folder on success, or a
/// `WallpaperGeneratorError` on failure.
pub(super) fn create_wallpaper_folder() -> Result<PathBuf, WallpaperGeneratorError> {
    let proj_dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .ok_or_else(|| WallpaperGeneratorError::OS("could not derive data_dir".to_string()))?;
    let path = proj_dirs.data_dir().join("Wallpapers");
    create_dir_all(&path).map_err(|e| WallpaperGeneratorError::OS(e.to_string()))?;
    Ok(path)
}

/// Deletes wallpapers from the "wallpapers" folder.
/// For each path, see: https://lib.rs/crates/directories
///
/// # Arguments
///
/// * `delete_all` - If true, deletes all wallpapers and the "astra_wallpapers" folder.
/// * `delete_dir` - If true, deletes the "astra_wallpapers" folder.
/// * `older_than` - If set, deletes wallpapers older than the specified frequency.
///
/// # Returns
///
/// A `Result` containing `()` on success, or a `WallpaperGeneratorError` on failure.
pub fn delete_wallpapers(
    config: &Config,
    delete_all: bool,
    delete_dir: bool,
    older_than: Option<&Frequency>,
) -> Result<(), WallpaperGeneratorError> {
    let path = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .map(|dirs| dirs.data_dir().join("Wallpapers"))
        .ok_or_else(|| {
            WallpaperGeneratorError::OS("could not derive wallpapers path".to_string())
        })?;
    config.print_if_verbose(format!("Deleting wallpapers from {}", path.display()).as_str());
    if delete_dir {
        remove_dir_all(&path).map_err(|e| WallpaperGeneratorError::OS(e.to_string()))?;
        config.print_if_verbose(
            format!(
                "Deleted all images and directory {} successfully",
                path.display()
            )
            .as_str(),
        );
    } else if delete_all {
        remove_dir_all(&path).map_err(|e| WallpaperGeneratorError::OS(e.to_string()))?;
        create_wallpaper_folder().map_err(|e| WallpaperGeneratorError::OS(e.to_string()))?;
        config.print_if_verbose(
            format!(
                "Deleted all images from directory {} successfully",
                path.display()
            )
            .as_str(),
        );
    } else if let Some(frequency) = older_than {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| WallpaperGeneratorError::OS(e.to_string()))?
            .as_secs();
        let older_than_sec = frequency.to_seconds();
        config.print_if_verbose(format!("Deleting images older than {}", &frequency).as_str());
        let oldest_timestamp_to_keep = now - older_than_sec;
        for entry in read_dir(&path).map_err(|e| WallpaperGeneratorError::OS(e.to_string()))? {
            let entry = entry.map_err(|e| WallpaperGeneratorError::OS(e.to_string()))?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            // string like spotlight_1640000000.png
            let timestamp_str =
                &file_name[file_name.rfind('_').map(|i| i + 1).unwrap_or(0)..file_name.len() - 4];
            match timestamp_str.parse::<u64>() {
                Ok(timestamp) => {
                    if timestamp < oldest_timestamp_to_keep {
                        remove_file(entry.path())
                            .map_err(|e| WallpaperGeneratorError::OS(e.to_string()))?;
                        config.print_if_verbose(
                            format!("Deleted image {} successfully", entry.path().display())
                                .as_str(),
                        );
                    }
                }
                Err(_) => {
                    config.print_if_verbose(format!(
                        "ERROR: Encountered file that is not an astra formatted image, skipping file... {}",
                        entry.path().display()
                    ).as_str());
                    continue;
                }
            };
        }
    }

    Ok(())
}

pub fn handle_generate_options(
    config: &Config,
    image_buf: &AstraImage,
    image: &Generator,
    no_save: bool,
    no_update: bool,
) -> Result<(), Box<dyn Error>> {
    // Handle options
    if !no_update {
        config.print_if_verbose(
            "NOTE: to update wallpaper, astra must save the image to astra_wallpapers folder.",
        );
        // Updating requires a saved image
        let saved_image_path = save_image(config, image, image_buf)?;
        // TODO: move verbose logs into OS implementations of update_wallpaper
        config.print_if_verbose("Updating wallpaper...");
        update_wallpaper(saved_image_path)?;
        config.print_if_verbose("Updated wallpaper");
    }
    // If no_update == false, we already saved the image as its required to update wallpaper
    if no_update && !no_save {
        let _ = save_image(config, image, image_buf)?;
    }
    Ok(())
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
pub(super) fn create_color_map(op: Operator, steps: usize, colors: &[[u8; 3]]) -> Vec<[u8; 3]> {
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
/// * `config` - A reference to the `Config` struct.
/// * `prefix` - A string to prepend to the file name.
/// * `image` - A reference to the `ImageBuffer` containing the image to save.
///
/// # Returns
///
/// A `Result` containing the path to the saved image on success, or a
/// `WallpaperGeneratorError` on failure.
pub fn save_image(
    config: &Config,
    generator: &Generator,
    image: &AstraImage,
) -> Result<PathBuf, WallpaperGeneratorError> {
    config.print_if_verbose("Saving image to astra_wallpapers folder...");
    let mut save_path = create_wallpaper_folder()?;

    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| WallpaperGeneratorError::OS(e.to_string()))?;

    save_path = save_path.join(format!("{}_{}.png", generator.prefix(), time.as_secs()));
    image
        .save(&save_path)
        .map_err(|_| WallpaperGeneratorError::ImageSave)?;
    config.print_if_verbose(
        format!(
            "Image saved to astra_wallpapers folder: {}",
            save_path.display()
        )
        .as_str(),
    );
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

/// Calculates the average color of the image.
///
/// Reference https://stackoverflow.com/questions/649454/what-is-the-best-way-to-average-two-colors-that-define-a-linear-gradient
/// Formula:
///
/// `(r_avg, g_avg, b_avg) = (sqrt((R_0^2 + ... + R_n^2) / n), sqrt((G_0^2 + ... + G_n^2) / n), sqrt((B_0^2 + ... + B_n^2) / n))`
///
/// # Arguments
///
/// * `image` - The image to calculate the average color of.
///
/// # Returns
///
/// The average color of the image.
pub fn average_color(image: &AstraImage) -> Rgb<u8> {
    let mut rgb_avg = (0.0, 0.0, 0.0);
    image.pixels().for_each(|color| {
        let r = color[0] as f64;
        let g = color[1] as f64;
        let b = color[2] as f64;
        rgb_avg.0 += r.powi(2);
        rgb_avg.1 += g.powi(2);
        rgb_avg.2 += b.powi(2);
    });
    Rgb::from([
        (rgb_avg.0 / image.len() as f64).sqrt() as u8,
        (rgb_avg.1 / image.len() as f64).sqrt() as u8,
        (rgb_avg.2 / image.len() as f64).sqrt() as u8,
    ])
}

// --- Utils ---

// --- Errors ---
#[derive(Debug, PartialEq)]
pub enum WallpaperGeneratorError {
    ImageGeneration(String),
    ImageSave,
    Network(String),
    OS(String),
    Parse(String),
}

impl fmt::Display for WallpaperGeneratorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WallpaperGeneratorError::ImageGeneration(msg) => {
                write!(f, "Image Generation Error: {}", msg)
            }
            WallpaperGeneratorError::ImageSave => {
                write!(f, "Failed to save image to file")
            }
            WallpaperGeneratorError::Network(msg) => {
                write!(f, "Network Error: {}", msg)
            }
            WallpaperGeneratorError::OS(msg) => {
                write!(f, "OS Error: {}", msg)
            }
            WallpaperGeneratorError::Parse(msg) => {
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
        let color_map = create_color_map(Operator::Gradient, 256, &[[255, 0, 0]]);
        assert_eq!(color_map.len(), 256);
        for color in color_map {
            assert_eq!(color, [255, 0, 0]);
        }
    }

    #[test]
    fn test_create_color_map_red_green() {
        let color_map = create_color_map(Operator::Gradient, 256, &[[255, 0, 0], [0, 255, 0]]);
        assert_eq!(color_map.len(), 256);
        assert_eq!(color_map[0], [255, 0, 0]);
        assert_eq!(color_map[255], [0, 255, 0]);
    }
}
// --- Tests ---
