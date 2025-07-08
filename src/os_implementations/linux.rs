use std::{fmt::Display, path::PathBuf, process::Command};
// --- OS specific code ---
/// CHecks if the user's OS is currently in dark mode
///
/// Tested on:
///   - Ubuntu 25.04 with Gnome Desktop
pub(crate) fn is_dark_mode_active() -> Result<bool, LinuxOSError> {
    let output = Command::new("gsettings")
        .arg("get")
        .arg("org.gnome.desktop.interface")
        .arg("color-scheme")
        .output()
        .map_err(|e| LinuxOSError::DarkModeError(e.to_string()))?;
    let output_str = String::from_utf8_lossy(&output.stdout);
    let dark_mode_enabled = match output_str.trim().to_lowercase().as_str() {
        "'prefer-dark'" => true,
        _ => false,
    };
    Ok(dark_mode_enabled)
}
pub(crate) fn get_screen_resolution() -> (u32, u32) {
    todo!("Implement for screen resolution on linux")
}

pub(crate) fn update_wallpaper(path: &str) -> () {
    todo!("Implement for updating wallpaper on linux")
}

pub(crate) fn path_to_desktop_folder() -> PathBuf {
    todo!("Implement path to desktop on linux")
}
// --- OS specific code ---

// --- Helper functions ---
// --- Helper functions ---

// --- Errors ---
#[derive(Debug, PartialEq)]
pub enum LinuxOSError {
    DarkModeError(String),
    HomeEnvVarNotFound,
    MainDisplayNotFound,
    ResolutionNotFound,
}

impl Display for LinuxOSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinuxOSError::DarkModeError(err_msg) => {
                write!(f, "Unable to determine dark mode status: {err_msg}")
            }
            LinuxOSError::HomeEnvVarNotFound => {
                write!(f, "Unable to find $HOME environment variable")
            }
            LinuxOSError::MainDisplayNotFound => write!(f, "Unable to determine main display"),
            LinuxOSError::ResolutionNotFound => {
                write!(f, "Unable to determine resolution of main display")
            }
        }
    }
}
// --- Errors ---
