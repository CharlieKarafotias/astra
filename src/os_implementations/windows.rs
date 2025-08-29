use std::{
    error::Error,
    os::{
        raw::c_void,
        windows::ffi::OsStrExt,
    },
    path::PathBuf,
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
/// Checks if the user's OS is currently in dark mode
///
/// # Errors
///
/// Returns a `WindowsError` with the `DarkModeError` variant if the command to determine
/// OS dark mode state cannot be executed. It can also return an error if the output
/// cannot be parsed.
pub(crate) fn is_dark_mode_active() -> Result<bool, WindowsError> {
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
///
/// # Errors
///
/// Returns a `WindowsError` with the `ScreenResolutionError` variant if the command to determine
/// screen resolution cannot be executed. It can also return an error if the output
/// cannot be parsed.
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

// --- OS specific code ---

// --- Errors ---
#[derive(Debug, PartialEq)]
pub enum WindowsError {
    DarkModeError(String),
    UpdateDesktopError(String),
}

impl std::fmt::Display for WindowsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowsError::DarkModeError(err) => {
                write!(f, "Unable to determine dark mode status: {err}")
            }
            WindowsError::UpdateDesktopError(err) => {
                write!(f, "Unable to update desktop wallpaper: {err}")
            }
        }
    }
}
impl Error for WindowsError {}
// --- Errors ---
