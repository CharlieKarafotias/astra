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
    let output = Command::new("schtasks")
        .args([
            "/create",
            "/sc",
            &sc.to_string(),
            "/tn",
            &format!("{QUALIFIER}_{ORGANIZATION}_{APPLICATION}"),
            "/tr",
            &curr_exe_path.to_string_lossy(),
            "/mo",
            &mo.to_string(),
            "/f",
        ])
        .output()
        .map_err(|e| WindowsError::CommandError(format!("schtasks create returned error: {e}")))?;
    if !output.status.success() {
        return Err(WindowsError::CommandError(format!(
            "schtasks failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(())
}

/// Uninstalls astra task when user config removes the frequency
/// [schtasks doc](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/schtasks-delete)
pub(in crate::os_implementations::windows) fn uninstall_astra_task() -> Result<(), WindowsError> {
    let task_name = format!("{QUALIFIER}_{ORGANIZATION}_{APPLICATION}");

    // Check if schtasks is available
    let is_available = Command::new("schtasks")
        .arg("/query")
        .arg("/tn")
        .arg(&task_name)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if !is_available {
        // Task doesn't exist or schtasks isn't available — skip uninstall
        return Ok(());
    }

    // Proceed with uninstall
    let output = Command::new("schtasks")
        .args(["/delete", "/tn", &task_name, "/f"])
        .output()
        .map_err(|e| WindowsError::CommandError(format!("schtasks delete returned error: {e}")))?;

    if !output.status.success() {
        return Err(WindowsError::CommandError(format!(
            "schtasks delete: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}
