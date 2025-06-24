use std::error::Error;
use std::path::PathBuf;
use std::process::Command;

// --- OS specific code ---
/// Checks if the user's OS is currently in dark mode
///
/// # Errors
///
/// Returns a `WindowsError` with the `DarkModeError` variant if the command to determine
/// OS dark mode state cannot be executed. It can also return an error if the output
/// cannot be parsed.
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
    let output_str =
        String::from_utf8(output.stdout).map_err(|e| WindowsError::DarkModeError(e.to_string()))?;
    let is_light_mode = output_str
        .trim()
        .parse::<i8>()
        .map_err(|e| WindowsError::DarkModeError(e.to_string()))?;
    Ok(is_light_mode == 0)
}

/// Retrieves the resolution of the largest display in pixels.
///
/// # Errors
///
/// Returns a `WindowsError` with the `ScreenResolutionError` variant if the command to determine
/// screen resolution cannot be executed. It can also return an error if the output
/// cannot be parsed.
pub(crate) fn get_screen_resolution() -> Result<(u32, u32), WindowsError> {
    let output = Command::new("powershell")
        .arg("-Command")
        .arg("Get-CimInstance")
        .arg("-ClassName")
        .arg("Win32_VideoController")
        .arg("|")
        .arg("Select-Object")
        .arg("VideoModeDescription")
        .output()
        .map_err(|e| WindowsError::ScreenResolutionError(e.to_string()))?;
    let (width, height) = parse_output(&String::from_utf8_lossy(&output.stdout))?;
    Ok((width, height))
}

pub(crate) fn update_wallpaper(path: PathBuf) -> () {
    todo!("Implement for updating wallpaper on windows")
}

pub(crate) fn path_to_desktop_folder() -> PathBuf {
    todo!("Implement path to desktop on windows")
}
// --- OS specific code ---

// --- Helper functions ---
/// Parses the output of the `Get-CimInstance` command to extract the resolution of the
/// largest display.
///
/// # Errors
///
/// Returns a `WindowsError` with the `ScreenResolutionError` variant if the output
/// cannot be parsed.
fn parse_output(output: &str) -> Result<(u32, u32), WindowsError> {
    let resolutions: Vec<(u32, u32)> = output
        .lines()
        .filter_map(|l| {
            let split: Vec<&str> = l.split('x').collect();
            if split.len() > 2 {
                let x = split[0].trim().parse::<u32>().ok()?;
                let y = split[1].trim().parse::<u32>().ok()?;
                Some((x, y))
            } else {
                None
            }
        })
        .collect();

    // Find the highest resolution
    let max_res = resolutions.iter().max_by_key(|res| res.0 * res.1);
    if let Some(max_res) = max_res {
        Ok(*max_res)
    } else {
        Err(WindowsError::ScreenResolutionError(
            "No resolutions returned".to_string(),
        ))
    }
}
// --- Helper functions ---

// --- Errors ---
#[derive(Debug, PartialEq)]
pub enum WindowsError {
    DarkModeError(String),
    ScreenResolutionError(String),
}

impl std::fmt::Display for WindowsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowsError::DarkModeError(err) => {
                write!(f, "Unable to determine dark mode status: {err}")
            }
            WindowsError::ScreenResolutionError(err) => {
                write!(f, "Unable to determine resolution of main display: {err}")
            }
        }
    }
}
impl Error for WindowsError {}
// --- Errors ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_output() {
        let input = "VideoModeDescription\n\
                    -----------------------\n\
                    2560 x 1440 x 4294967296 colors\n\
                    1920 x 1080 x 4294967296 colors\n\
                    1280 x 720 x 4294967296 colors";

        let result = parse_output(input).unwrap();
        assert_eq!(result, (2560, 1440));
    }

    #[test]
    fn test_parse_output_single_resolution() {
        let input = "VideoModeDescription\n\
                    -----------------------\n\
                    3840 x 2160 x 4294967296 colors";

        let result = parse_output(input).unwrap();
        assert_eq!(result, (3840, 2160));
    }

    #[test]
    fn test_parse_output_invalid_format() {
        let input = "Invalid format\n\
                    -----------------------\n\
                    Not a resolution";

        let result = parse_output(input);
        assert!(matches!(
            result,
            Err(WindowsError::ScreenResolutionError(_))
        ));
    }
}
