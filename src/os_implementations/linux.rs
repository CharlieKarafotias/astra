use std::{fmt::Display, path::PathBuf, process::Command};
// --- OS specific code ---
/// CHecks if the user's OS is currently in dark mode
///
/// Tested on:
///   - Ubuntu 25.04 with Gnome Desktop
pub(crate) fn is_dark_mode_active() -> Result<bool, LinuxOSError> {
    // TODO: add support for other linux distros (non gnome based)
    let output = Command::new("gsettings")
        .arg("get")
        .arg("org.gnome.desktop.interface")
        .arg("color-scheme")
        .output()
        .map_err(|e| LinuxOSError::DarkModeError(e.to_string()))?;
    let output_str = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();
    Ok(output_str.contains("prefer-dark"))
}
pub(crate) fn get_screen_resolution() -> Result<(u32, u32), LinuxOSError> {
    let output = Command::new("xrandr")
        .arg("--current")
        .arg("|")
        .arg("grep")
        .arg("oP")
        .arg("'current \\K[0-9]+ x [0-9]+'")
        .output()
        .map_err(|e| LinuxOSError::ResolutionNotFound(e.to_string()))?;
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .split_once('x')
        .map(|(w, h)| (w.parse::<u32>().map_err(|e| LinuxOSError::ParseError(e.to_string())), h.parse::<u32>().map_err(|e| LinuxOSError::ParseError(e.to_string()))))
        .ok_or(|e| LinuxOSError::ResolutionNotFound(e.to_string()))
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
    ParseError(String),
    ResolutionNotFound(String),
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
            LinuxOSError::ParseError(err_msg) => {
                write!(f, "Unable to parse output: {err_msg}")
            }
            LinuxOSError::ResolutionNotFound(err_msg) => {
                write!(f, "Unable to determine resolution of main display: {err_msg}")
            }
        }
    }
}
// --- Errors ---
