use std::{error::Error, fmt::Display, path::PathBuf, process::Command};

// --- OS specific code ---
/// Checks if the user's OS is currently in dark mode
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
    let output_str = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_lowercase();
    Ok(output_str.contains("prefer-dark"))
}

/// Gets the resolution of the primary display. This relies on the `xrandr` command to
/// determine the resolution.
///
/// # Errors
///
/// Returns a `LinuxOSError` with the `ResolutionNotFound` variant if the command to determine
/// screen resolution cannot be executed. It can also return an error if the output
/// cannot be parsed.
pub(crate) fn get_screen_resolution() -> Result<(u32, u32), LinuxOSError> {
    // First, get the primary display name
    let output = Command::new("xrandr")
        .arg("--current")
        .output()
        .map_err(|e| LinuxOSError::ResolutionNotFound(e.to_string()))?;

    // Parse the output to find the current resolution
    let output_str = String::from_utf8_lossy(&output.stdout);

    // Look for the primary display line with resolution
    for line in output_str.lines() {
        if line.contains("connected primary") {
            if let Some(resolution_part) = line.split_whitespace().nth(3) {
                let resolution = resolution_part.trim_matches('+');
                if let Some((w, h)) = resolution.split_once('x') {
                    let width = w
                        .parse::<u32>()
                        .map_err(|e| LinuxOSError::ParseError(e.to_string()))?;
                    let height = h
                        .split('+')
                        .next()
                        .unwrap_or(h)
                        .parse::<u32>()
                        .map_err(|e| LinuxOSError::ParseError(e.to_string()))?;
                    return Ok((width, height));
                }
            }
        }
    }

    Err(LinuxOSError::ResolutionNotFound(
        "Could not determine screen resolution".to_string(),
    ))
}

/// Sets the wallpaper to the given path. This relies on the `gsettings` command to
/// set the wallpaper.
///
/// This function has been tested on:
///   - Ubuntu 25.04 with Gnome Desktop
///
/// # Errors
///
/// Returns a `LinuxOSError` with the `CommandError` variant if the `gsettings` command
/// cannot be executed.
pub(crate) fn update_wallpaper(path: PathBuf) -> Result<(), LinuxOSError> {
    // TODO: add support for other linux distros (non gnome based)
    let output = Command::new("gsettings")
        .arg("set")
        .arg("org.gnome.desktop.background")
        .arg("picture-uri")
        .arg(format!("\"file:///{path}\""))
        .output()
        .map_err(|e| LinuxOSError::CommandError(e.to_string()))?;
    Ok(())
}

/// Returns the path to the user's desktop folder. This relies on the `xdg-user-dir` command to
/// determine the path.
///
/// # Errors
///
/// Returns a `LinuxOSError` with the `CommandError` variant if the `xdg-user-dir` command
/// cannot be executed.
pub(crate) fn path_to_desktop_folder() -> Result<PathBuf, LinuxOSError> {
    let output = Command::new("xdg-user-dir")
        .arg("DESKTOP")
        .output()
        .map_err(|e| LinuxOSError::CommandError(e.to_string()))?;
    let desktop_path = String::from_utf8_lossy(&output.stdout);
    Ok(PathBuf::from(desktop_path.trim()))
}
// --- OS specific code ---

// --- Helper functions ---
// --- Helper functions ---

// --- Errors ---
#[derive(Debug, PartialEq)]
pub enum LinuxOSError {
    CommandError(String),
    DarkModeError(String),
    ParseError(String),
    ResolutionNotFound(String),
}

impl Display for LinuxOSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinuxOSError::CommandError(err_msg) => {
                write!(f, "Unable to execute command: {err_msg}")
            }
            LinuxOSError::DarkModeError(err_msg) => {
                write!(f, "Unable to determine dark mode status: {err_msg}")
            }
            LinuxOSError::ParseError(err_msg) => {
                write!(f, "Unable to parse output: {err_msg}")
            }
            LinuxOSError::ResolutionNotFound(err_msg) => {
                write!(
                    f,
                    "Unable to determine resolution of main display: {err_msg}"
                )
            }
        }
    }
}

impl Error for LinuxOSError {}
// --- Errors ---

// --- Tests ---
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolution() {
        // This is example output of xrandr
        let xrandr_output = r#"Screen 0: minimum 16 x 16, current 1920 x 1080, maximum 32767 x 32767
            HDMI-1 connected primary 1920x1080+0+0 (normal left inverted right x axis y axis) 800mm x 340mm
               1920x1080     59.96*+
               1440x1080     59.99
               1400x1050     59.98
               1280x1024     59.89
               1280x960      59.94
               1152x864      59.96
               1024x768      59.92
               800x600       59.86
               640x480       59.38
               320x240       59.29
               1680x1050     59.95
               1440x900      59.89
               1280x800      59.81
               1152x720      59.97
               960x600       59.63
               928x580       59.88
               800x500       59.50
               768x480       59.90
               720x480       59.71
               640x400       59.95
               320x200       58.14
               1600x900      59.95
               1368x768      59.88
               1280x720      59.86
               1024x576      59.90
               864x486       59.92
               720x400       59.27
               640x350       59.28
        "#;

        assert_eq!(get_screen_resolution().unwrap(), (1920, 1080));
    }
}
