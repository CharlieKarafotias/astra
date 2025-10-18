use super::super::MacOSError;
use std::{path::PathBuf, process::Command};

/// Run the bootstrap command using provided plist file.
/// This command is useful to ensure new job runs prior to system
/// restart.
pub(in crate::os_implementations::macos) fn launchctl_bootstrap_astra(
    plist_path: &PathBuf,
) -> Result<(), MacOSError> {
    let user_id = get_user_id()?;
    Command::new("launchctl")
        .arg("bootstrap")
        .arg(format!("gui/{user_id}"))
        .arg(plist_path)
        .output()
        .map_err(|e| MacOSError::Launchctl(e.to_string()))?;
    Ok(())
}

/// Run the bootout command using provided plist file.
/// This command is useful to ensure Job does not continue
/// to run when user updates configuration file.
pub(in crate::os_implementations::macos) fn launchctl_bootout_astra(
    plist_path: &PathBuf,
) -> Result<(), MacOSError> {
    let user_id = get_user_id()?;
    Command::new("launchctl")
        .arg("bootout")
        .arg(format!("gui/{user_id}"))
        .arg(plist_path)
        .output()
        .map_err(|e| MacOSError::Launchctl(e.to_string()))?;
    Ok(())
}

/// A helper function that gets the users current id
///
/// Errors:
/// - Will error if user id command fails
fn get_user_id() -> Result<String, MacOSError> {
    let user_id_vec = Command::new("id")
        .arg("-u")
        .output()
        .map_err(|e| MacOSError::OS(format!("unable to get user id: {e}")))?
        .stdout;
    let mut user_id = String::from_utf8_lossy(&user_id_vec).to_string();
    user_id = user_id.trim().to_string();
    Ok(user_id)
}
