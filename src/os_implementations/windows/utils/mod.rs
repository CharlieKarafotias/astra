use super::super::super::Config;
use super::{WindowsError, install_astra_task, uninstall_astra_task};
use std::{
    os::{raw::c_void, windows::ffi::OsStrExt},
    path::PathBuf,
    process::Command,
};
use windows::{
    Win32::{
        System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW},
        UI::WindowsAndMessaging::{
            GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN, SPI_SETDESKWALLPAPER, SPIF_SENDCHANGE,
            SPIF_UPDATEINIFILE, SystemParametersInfoW,
        },
    },
    core::PCWSTR,
};

// --- OS specific code ---
/// Sync wallpaper functionality for Windows
/// TODO: Implement get_current_wallpapers() - detect current wallpaper on each monitor
/// TODO: Implement has_astra_wallpaper() - check if any monitor has an Astra wallpaper
/// TODO: Implement get_astra_wallpaper_path() - get the path of the Astra wallpaper
/// TODO: Implement sync_wallpapers() - sync all monitors to the same wallpaper
/// 
/// Notes:
/// - On Windows, getting current wallpaper per monitor is complex
/// - Windows 10/11 use a single wallpaper per session
/// - Multiple monitors share the same wallpaper
/// - Getting the wallpaper path requires reading from registry or using API calls
/// 
/// Implementation approach:
/// 1. Use Windows API to get current wallpaper path
/// 2. Check if wallpaper path ends with "astra_1.png" or "astra_2.png"
/// 3. If Astra wallpaper detected, sync all monitors (they share wallpaper anyway)
/// 4. If no Astra wallpaper, log warning and return

pub fn get_current_wallpapers() -> Result<Vec<String>, WindowsError> {
    // TODO: Implement getting current wallpapers
    // Note: On Windows 10/11, all monitors share the same wallpaper
    // Use SystemParametersInfo with SPI_GETDESKWALLPAPER
    todo!("Implement get_current_wallpapers() for Windows")
}

pub fn has_astra_wallpaper() -> Result<bool, WindowsError> {
    // TODO: Implement checking for Astra wallpaper
    todo!("Implement has_astra_wallpaper() for Windows")
}

pub fn get_astra_wallpaper_path() -> Result<Option<PathBuf>, WindowsError> {
    // TODO: Implement getting Astra wallpaper path
    todo!("Implement get_astra_wallpaper_path() for Windows")
}

/// Synchronizes all monitors to the same wallpaper
/// If an Astra wallpaper is detected, all monitors are synced to it
/// If no Astra wallpaper is detected, a warning is logged and no changes are made
pub fn sync_wallpapers(config: &Config) -> Result<(), WindowsError> {
    // TODO: Implement sync functionality
    // 
    // Note: On Windows 10/11, all monitors share the same wallpaper
    // So sync is not strictly necessary, but we can still check and ensure
    // the wallpaper is set correctly if an Astra wallpaper is detected
    //
    // Implementation approach:
    // 1. Check if any monitor has an Astra wallpaper
    // 2. If yes, ensure it's set on all monitors (they share wallpaper anyway)
    // 3. If no Astra wallpaper, log warning and return
    todo!(
        "Implement sync_wallpapers() for Windows - Windows monitors share wallpaper"
    )
}

/// Sync wallpaper functionality for the frequency handler
/// This function is called after the wallpaper check to ensure all monitors
/// are synced if an Astra wallpaper is detected
pub fn handle_sync(config: &Config) -> Result<(), WindowsError> {
    // TODO: Call sync_wallpapers() when implemented for Windows
    // For now, just log that sync is not yet implemented
    config.print_if_verbose("Windows sync not yet implemented");
    Ok(())
}

/// Checks if the user's OS is currently in dark mode
pub fn is_dark_mode_active() -> Result<bool, WindowsError> {
    let mut data: u32 = 0;
    let mut data_size = std::mem::size_of::<u32>() as u32;

    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            PCWSTR::from(windows::core::w!(
                "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"
            )),
            PCWSTR::from(windows::core::w!("SystemUsesLightTheme")),
            RRF_RT_REG_DWORD,
            None,
            Some(&mut data as *mut _ as *mut _),
            Some(&mut data_size),
        )
    };
    status
        .ok()
        .map_err(|e| WindowsError::DarkModeError(format!("RegGetValueW failed: {e}")))?;
    Ok(data == 0) // 0 = dark mode, 1 = light mode
}

/// Retrieves the resolution of the largest display in pixels.
pub(crate) fn get_screen_resolution() -> Result<(u32, u32), WindowsError> {
    let width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let height = unsafe { GetSystemMetrics(SM_CYSCREEN) };
    Ok((width as u32, height as u32))
}

pub(crate) fn update_wallpaper(path: PathBuf) -> Result<(), WindowsError> {
    let widestr: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let result = unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            Some(widestr.as_ptr() as *mut c_void),
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        )
    };

    result
        .map_err(|e| WindowsError::UpdateDesktopError(format!("SystemParametersInfoW failed: {e}")))
}

/// Opens the given file in the user's default editor. This function relies on the start
/// command to open the file.
pub(crate) fn open_editor(config: &Config, path: PathBuf) -> Result<(), WindowsError> {
    config.print_if_verbose("Using default editor");
    Command::new("powershell")
        .arg("-Command")
        .arg("start")
        .arg(path)
        .output()
        .map_err(|e| WindowsError::OpenEditorError(format!("Failed to open editor: {e}")))?;
    Ok(())
}

/// CRUD operator function for interfacing with Windows task scheduler service
pub(crate) fn handle_frequency(config: &Config) -> Result<bool, WindowsError> {
    if let Some(frequency) = config.frequency() {
        install_astra_task(frequency)?;
    } else {
        uninstall_astra_task()?;
    }
    Ok(true)
}
