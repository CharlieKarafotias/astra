use super::super::super::super::Frequency;
use super::super::MacOSError;
use crate::constants::{APPLICATION, ORGANIZATION, QUALIFIER};
use directories::UserDirs;
use std::path::PathBuf;

/// A helper function that generates the plist file path
/// The path will be ~/Library/LaunchAgents/dev.CharlieKarafotias.astra.plist
///
/// # Errors
/// - Will error is UserDirs is None. This should NEVER happen!
pub(in crate::os_implementations::macos) fn gen_plist_path() -> Result<PathBuf, MacOSError> {
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
pub(in crate::os_implementations::macos) fn gen_plist_for_astra(
    frequency: &Frequency,
) -> Result<String, MacOSError> {
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

// TODO: v1.1.0 - add tests for both plist funcs here
