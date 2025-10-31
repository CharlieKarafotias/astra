use super::super::super::Frequency;
use super::super::MacOSError;
use crate::constants::{APPLICATION, ORGANIZATION, QUALIFIER};
use directories::UserDirs;
use std::{fs, path::PathBuf, process::Command};

/// A helper function that generates the plist file path
/// The path will be ~/Library/LaunchAgents/dev.CharlieKarafotias.astra.plist
///
/// # Errors
/// - Will error is UserDirs is None. This should NEVER happen!
fn gen_plist_path() -> Result<PathBuf, MacOSError> {
    let mut path_to_astra_plist = UserDirs::new()
        .ok_or(MacOSError::OS("home directory not defined".to_string()))?
        .home_dir()
        .to_path_buf();
    path_to_astra_plist.push("Library");
    path_to_astra_plist.push("LaunchAgents");
    path_to_astra_plist.push(format!("{QUALIFIER}.{ORGANIZATION}.{APPLICATION}.plist"));
    Ok(path_to_astra_plist)
}

/// A helper function to generate the contents of the astra program's plist file.
/// The file contents is used by handle_frequency function to create/update the associated astra
/// task in launchd
///
/// Resource: https://launchd.info/
fn gen_plist_for_astra(frequency: &Frequency) -> Result<String, MacOSError> {
    let curr_exe_path: String = std::env::current_exe()
        .map_err(|_| MacOSError::OS("failed to derive current executable path".to_string()))?
        .into_os_string()
        .into_string()
        .map_err(|_| MacOSError::StringConversion)?;
    let file_contents = format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
    <dict>
        <key>Label</key>
        <string>{}.{}.{}</string>
        <key>Program</key>
        <string>{}</string>
        <key>StartInterval</key>
        <integer>{}</integer>
        <key>RunAtLoad</key>
        <true/>
    </dict>
</plist>",
        QUALIFIER,
        ORGANIZATION,
        APPLICATION,
        curr_exe_path,
        frequency.to_seconds()
    );
    Ok(file_contents)
}

/// Run the bootstrap command using provided plist file.
/// This command is useful to ensure new job runs prior to system
/// restart.
fn launchctl_bootstrap_astra(plist_path: &PathBuf) -> Result<(), MacOSError> {
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
fn launchctl_bootout_astra(plist_path: &PathBuf) -> Result<(), MacOSError> {
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

pub(in crate::os_implementations::macos) fn install_astra_freq_with_launchctl(
    frequency: &Frequency,
) -> Result<(), MacOSError> {
    let path_to_astra_plist = gen_plist_path()?;
    let file_contents = gen_plist_for_astra(frequency)?;
    // NOTE: - it is a known "issue" that you must turn off frequency first, run astra,
    // then add new frequency update for launchctl to accept changes
    fs::write(&path_to_astra_plist, file_contents).map_err(|err_msg| {
        MacOSError::OS(format!("failed to create/update plist file: {err_msg}"))
    })?;
    launchctl_bootstrap_astra(&path_to_astra_plist)?;
    Ok(())
}

pub(in crate::os_implementations::macos) fn uninstall_astra_freq_from_launchctl()
-> Result<(), MacOSError> {
    let path_to_astra_plist = gen_plist_path()?;
    launchctl_bootout_astra(&path_to_astra_plist)?;
    if path_to_astra_plist.exists() {
        fs::remove_file(&path_to_astra_plist)
            .map_err(|err_msg| MacOSError::OS(format!("failed to delete plist file: {err_msg}")))?;
    }
    Ok(())
}
