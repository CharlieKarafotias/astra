use std::error::Error;
use std::path::PathBuf;
use std::process::Command;

// --- OS specific code ---
pub(crate) fn is_dark_mode_active() -> Result<bool, WindowsError> {
    let output = Command::new("powershell")
        .arg("-Command")
        .arg("Get-ItemPropertyValue")
        .arg("-Path")
        .arg("HKCU:\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize")
        .arg("-Name")
        .arg("SystemUsesLightTheme")
        .output()
        .map_err(|e| WindowsError::DarkModeError(e.to_string()))?;
    let output_str = String::from_utf8(output.stdout).map_err(|e| WindowsError::DarkModeError(e.to_string()))?;
    let is_light_mode = output_str.trim().parse::<i8>().map_err(|e| WindowsError::DarkModeError(e.to_string()))?;
    Ok(is_light_mode == 0)
    
}
pub(crate) fn get_screen_resolution() -> (u32, u32) {
    todo!("Implement for screen resolution on windows")
}

pub(crate) fn update_wallpaper(path: PathBuf) -> () {
    todo!("Implement for updating wallpaper on windows")
}

pub(crate) fn path_to_desktop_folder() -> PathBuf {
    todo!("Implement path to desktop on windows")
}
// --- OS specific code ---

// --- Helper functions ---
// --- Helper functions ---

// --- Errors ---
#[derive(Debug, PartialEq)]
pub enum WindowsError {
    DarkModeError(String),
}

impl std::fmt::Display for WindowsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowsError::DarkModeError(err) => write!(f, "Unable to determine dark mode status: {err}"),
        }
    }
}
impl Error for WindowsError {}
// --- Errors ---
