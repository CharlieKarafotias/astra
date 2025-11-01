use super::super::super::Frequency;
use super::WindowsError;
use crate::constants::{APPLICATION, ORGANIZATION, QUALIFIER};
use std::{env::current_exe, process::Command};

/// Installs astra task when user config includes a frequency
/// Limitations:
///   - Due to schtasks create not supporting seconds and higher than 1 year range, user
///   frequencies that include anything under 60s will default to 1m and anything beyond 1y will
///   default to 12 months in schtask
/// [schtasks doc](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/schtasks-create)
pub(in crate::os_implementations::windows) fn install_astra_task(
    frequency: &Frequency,
) -> Result<(), WindowsError> {
    let curr_exe_path = current_exe().map_err(|e| WindowsError::ExecutablePath(e.to_string()))?;
    let (mo, sc) = frequency.as_task_scheduler_components();
    Command::new("schtasks")
        .args([
            "/create",
            "/sc",
            &sc.to_string(),
            "/tn",
            &format!("\"{QUALIFIER}.{ORGANIZATION}.{APPLICATION}\""),
            "/tr",
            &curr_exe_path.to_string_lossy().to_string(),
            "/mo",
            &mo.to_string(),
            "/it",
            "/f",
        ])
        .status()
        .map_err(|e| WindowsError::CommandError(format!("schtasks create returned error: {e}")))?;
    Ok(())
}

/// Uninstalls astra task when user config removes the frequency
/// [schtasks doc](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/schtasks-delete)
pub(in crate::os_implementations::windows) fn uninstall_astra_task() -> Result<(), WindowsError> {
    Command::new("schtasks")
        .args([
            "/delete",
            "/tn",
            &format!("\"{QUALIFIER}.{ORGANIZATION}.{APPLICATION}\""),
            "/f",
        ])
        .status()
        .map_err(|e| WindowsError::CommandError(format!("schtasks delete returned error: {e}")))?;
    Ok(())
}
