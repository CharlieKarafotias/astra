use super::os_implementations::path_to_desktop_folder;
use std::error::Error;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

mod fractals;
pub use fractals::*;

// --- Utils ---

/// Creates a folder named "astra_wallpapers" on the desktop.
///
/// # Returns
///
/// A `Result` containing the path to the created folder on success, or a
/// `WallpaperGeneratorError` on failure.
pub(crate) fn create_wallpaper_folder() -> Result<PathBuf, WallpaperGeneratorError> {
    println!("Preparing astra_wallpaper folder on desktop...");
    let path = path_to_desktop_folder()
        .map_err(|e| WallpaperGeneratorError::OSError(e.to_string()))?
        .join("astra_wallpapers");
    create_dir_all(&path).map_err(|e| WallpaperGeneratorError::OSError(e.to_string()))?;
    Ok(path)
}

/// Saves the given image to a file in the desktop wallpaper folder.
///
/// The file is named using the current UNIX timestamp to ensure uniqueness.
/// The image is saved in PNG format.
///
/// # Arguments
///
/// * `image` - A reference to the `ImageBuffer` containing the image to save.
///
/// # Returns
///
/// A `Result` containing the path to the saved image on success, or a
/// `WallpaperGeneratorError` on failure.
pub(crate) fn save_image(
    image: &image::ImageBuffer<image::Rgb<u8>, Vec<u8>>,
) -> Result<PathBuf, WallpaperGeneratorError> {
    let mut save_path = create_wallpaper_folder()?;

    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| WallpaperGeneratorError::OSError(e.to_string()))?;

    save_path = save_path.join(format!("julia_{}.png", time.as_secs().to_string()));

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
pub(crate) fn scale_image(
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
    DesktopWallpaperFolderCreationError,
    ImageSaveError,
    OSError(String),
}

impl std::fmt::Display for WallpaperGeneratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WallpaperGeneratorError::DesktopWallpaperFolderCreationError => {
                write!(f, "Unable to create wallpaper folder on desktop")
            }
            WallpaperGeneratorError::ImageSaveError => {
                write!(f, "Failed to save image to file")
            }
            WallpaperGeneratorError::OSError(msg) => {
                write!(f, "OS Error: {}", msg)
            }
        }
    }
}

impl Error for WallpaperGeneratorError {}
// --- Errors ---
