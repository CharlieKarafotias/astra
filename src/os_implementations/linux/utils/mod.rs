use super::super::Config;
use std::{
    env::var,
    error::Error,
    fmt::Display,
    path::PathBuf,
    process::{Command, Stdio},
};

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
    let picture_uri_arg = if is_dark_mode_active()? {
        "picture-uri-dark"
    } else {
        "picture-uri"
    };
    Command::new("gsettings")
        .arg("set")
        .arg("org.gnome.desktop.background")
        .arg(picture_uri_arg)
        .arg(path)
        .output()
        .map_err(|e| LinuxOSError::CommandError(e.to_string()))?;
    Ok(())
}

/// Opens the given file in the user's default editor.
/// This function will first check the `EDITOR` environment variable, and if it is not set,
/// it will default to using `vim`.
///
/// # Errors
/// - Returns a `LinuxOSError` with the `OpenEditorError` variant if the command to open the
/// file cannot be executed for any reason.
pub(crate) fn open_editor(config: &Config, path: PathBuf) -> Result<(), LinuxOSError> {
    let editor = var("EDITOR").unwrap_or("vim".to_string());
    config.print_if_verbose(&format!("Using editor: {}", editor));
    let status = Command::new(&editor)
        .arg(path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|_| LinuxOSError::OpenEditorError)?;

    if !status.success() {
        return Err(LinuxOSError::OpenEditorError);
    }
    Ok(())
}

pub(crate) fn handle_frequency(config: &Config) -> Result<(), LinuxOSError> {
    todo!("implement me")
}

// --- OS specific code ---

// --- Helper functions ---
// --- Helper functions ---

// --- Errors ---
#[derive(Debug, PartialEq)]
pub enum LinuxOSError {
    CommandError(String),
    DarkModeError(String),
    OpenEditorError,
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
            LinuxOSError::OpenEditorError => {
                write!(f, "Unable to open editor")
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
