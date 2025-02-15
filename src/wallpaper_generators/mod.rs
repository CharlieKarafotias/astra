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
