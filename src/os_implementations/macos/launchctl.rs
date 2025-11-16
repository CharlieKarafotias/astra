use super::super::MacOSError;
use crate::constants::{APPLICATION, MAC_OS_LAUNCHCTL_INTERVAL, ORGANIZATION, QUALIFIER};
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
/// launchd does not support persistent durations. As such, the astra job instead runs every
/// MAC_OS_LAUNCHCTL_INTERVAL seconds. It is then handle_frequency function's job to determine
/// if the right amount of time has elapsed and if wallpaper should be updated by astra.
///
/// Resource: https://launchd.info/
fn gen_plist_for_astra() -> Result<String, MacOSError> {
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
        MAC_OS_LAUNCHCTL_INTERVAL,
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

/// Runs the print command to search for existance of Astra job.
/// IF found, will return the run interval
/// IF NOT found, will return None
pub(in crate::os_implementations::macos) fn launchctl_check_existance_of_astra_job()
-> Result<Option<u64>, MacOSError> {
    let user_id = get_user_id()?;
    let output = Command::new("launchctl")
        .arg("print")
        .arg(format!(
            "gui/{user_id}/{QUALIFIER}.{ORGANIZATION}.{APPLICATION}"
        ))
        .output()
        .map_err(|e| MacOSError::Launchctl(e.to_string()))?;
    // NOTE: any error status is interpreted as job doesn't exist and should be readded by
    // handle_frequency
    if !output.status.success() {
        return Ok(None);
    }
    Ok(extract_interval(&output.stdout))
}

// A helper function that takes the output of launchctl print command and returns the
// value of run interval if it exists in the output
fn extract_interval(output: &[u8]) -> Option<u64> {
    String::from_utf8_lossy(output)
        .lines()
        .find(|l| l.contains("run interval"))
        // example line: run interval = 600 seconds
        .and_then(|l| l.splitn(2, '=').nth(1))
        .and_then(|l| l.trim().split_whitespace().next())
        .and_then(|n| n.parse::<u64>().ok())
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

/// A helper function to create or update the plist file responsible for running astra program
/// at a set interval (10min).
///
/// The function will:
///   1. Check for the existance of the Astra job in launchctl.
///   2. IF job exists, check that interval is 10minutes. If so then exits.
///   3. IF update required, generates the plist file and writes it
///   4. Then calls bootstrap to execute astra
pub(in crate::os_implementations::macos) fn launchctl_install_astra_freq() -> Result<(), MacOSError>
{
    let path_to_astra_plist = gen_plist_path()?;
    let file_contents = gen_plist_for_astra()?;
    fs::write(&path_to_astra_plist, file_contents).map_err(|err_msg| {
        MacOSError::OS(format!("failed to create/update plist file: {err_msg}"))
    })?;
    launchctl_bootstrap_astra(&path_to_astra_plist)?;
    Ok(())
}

pub(in crate::os_implementations::macos) fn launchctl_uninstall_astra_freq()
-> Result<(), MacOSError> {
    let path_to_astra_plist = gen_plist_path()?;
    launchctl_bootout_astra(&path_to_astra_plist)?;
    if path_to_astra_plist.exists() {
        fs::remove_file(&path_to_astra_plist)
            .map_err(|err_msg| MacOSError::OS(format!("failed to delete plist file: {err_msg}")))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_interval_from_launchctl_output() {
        // A minimal chunk of sample output including the line we need.
        // You can include more if you want, but only the "run interval" line matters.
        let sample = r#"
gui/501/dev.CharlieKarafotias.Astra = {
    active count = 0
    path = /Users/someuser/Library/LaunchAgents/dev.CharlieKarafotias.Astra.plist
    type = LaunchAgent

    run interval = 86400 seconds

    properties = runatload | penalty box | system service
}
"#;

        let secs = extract_interval(sample.as_bytes());

        assert_eq!(secs, Some(86400));
    }

    #[test]
    fn test_extract_interval_from_launchctl_output_when_no_interval() {
        let sample = r#"
gui/501/dev.CharlieKarafotias.Astra = {
    active count = 0
    path = /Users/someuser/Library/LaunchAgents/dev.CharlieKarafotias.Astra.plist
    type = LaunchAgent
    properties = runatload | penalty box | system service
}
"#;

        let secs = extract_interval(sample.as_bytes());

        assert_eq!(secs, None)
    }
}
