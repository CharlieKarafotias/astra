use super::super::super::Config;
use super::{LinuxOSError, install_astra_service_and_timer, uninstall_astra_serivice_and_timer};
use std::{
    env::var,
    path::PathBuf,
    process::{Command, Stdio},
};

// --- OS specific code ---
/// Checks if the user's OS is currently in dark mode
///
/// Tested on:
///   - Ubuntu 25.04 with Gnome Desktop
pub fn is_dark_mode_active() -> Result<bool, LinuxOSError> {
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

/// Sync wallpaper functionality for Linux
/// TODO: Implement get_current_wallpapers() - detect current wallpaper on each monitor
/// TODO: Implement has_astra_wallpaper() - check if any monitor has an Astra wallpaper
/// TODO: Implement get_astra_wallpaper_path() - get the path of the Astra wallpaper
/// TODO: Implement sync_wallpapers() - sync all monitors to the same wallpaper
///
/// Notes:
/// - On Linux, getting current wallpaper per monitor is complex and distro-dependent
/// - GNOME uses gsettings with picture-uri/picture-uri-dark
/// - KDE uses kwriteconfig5 with kcm_wallpaper
/// - Other DEs have their own methods
///
/// Implementation approach:
/// 1. Use xprop to get current wallpaper: xprop -root _NET_WM_BACKGROUND_FILE
/// 2. Parse wallpaper URI to get path
/// 3. Check if path ends with "astra_1.png" or "astra_2.png"
/// 4. If Astra wallpaper detected, sync all monitors using gsettings

pub fn get_current_wallpapers() -> Result<Vec<String>, LinuxOSError> {
    // TODO: Implement getting current wallpapers
    // Approach: Use xprop to get wallpaper info
    // Command: xprop -root _NET_WM_BACKGROUND_FILE
    // This is complex and may not work on all Linux distributions
    todo!("Implement get_current_wallpapers() for Linux - requires distro-specific approach")
}

pub fn has_astra_wallpaper() -> Result<bool, LinuxOSError> {
    // TODO: Implement checking for Astra wallpaper
    todo!("Implement has_astra_wallpaper() for Linux")
}

pub fn get_astra_wallpaper_path() -> Result<Option<PathBuf>, LinuxOSError> {
    // TODO: Implement getting Astra wallpaper path
    todo!("Implement get_astra_wallpaper_path() for Linux")
}

/// Synchronizes all monitors to the same wallpaper
/// If an Astra wallpaper is detected on any monitor, all monitors are synced to it
/// If no Astra wallpaper is detected, a warning is logged and no changes are made
pub fn sync_wallpapers(config: &Config) -> Result<(), LinuxOSError> {
    // TODO: Implement sync functionality
    //
    // Implementation approach:
    // 1. Check if any monitor has an Astra wallpaper
    // 2. If yes, get the path and use gsettings to set it on all monitors
    // 3. If no Astra wallpaper, log warning and return
    //
    // Note: On Linux, syncing wallpapers across monitors is not straightforward
    // as each monitor may have its own wallpaper settings
    todo!("Implement sync_wallpapers() for Linux - requires distro-specific approach")
}

/// Gets the resolution of the primary display. This relies on the `xrandr` command to
/// determine the resolution.
///
/// # Errors
///
/// Returns a `LinuxOSError` with the `ResolutionNotFound` variant if the command to determine
/// screen resolution cannot be executed. It can also return an error if the output
/// cannot be parsed.
pub fn get_screen_resolution() -> Result<(u32, u32), LinuxOSError> {
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
pub fn update_wallpaper(path: PathBuf) -> Result<(), LinuxOSError> {
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
pub fn open_editor(config: &Config, path: PathBuf) -> Result<(), LinuxOSError> {
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

/// CRUD operator function for interfacing with systemd system in Linux
///
/// This function takes in the configuration struct and checks if user config contains a frequency
/// key/value.
///
/// - If key/value is defined, take the frequency and ensure astra service/timer is created/updated
/// - If key/value is not defined, ensure the astra service/timer file is deleted (if it exists)
pub fn handle_frequency(config: &Config) -> Result<bool, LinuxOSError> {
    if let Some(frequency) = config.frequency() {
        install_astra_service_and_timer(frequency)?;
    } else {
        uninstall_astra_serivice_and_timer()?;
    }
    Ok(true)
}
