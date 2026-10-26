use super::super::super::Config;
use super::{
    MacOSError, launchctl_check_existence_of_astra_job, launchctl_install_astra_freq,
    launchctl_uninstall_astra_freq,
};
use crate::constants::{APPLICATION, MAC_OS_LAUNCHCTL_INTERVAL, ORGANIZATION, QUALIFIER};
use directories::{ProjectDirs, UserDirs};
use objc2::MainThreadMarker;
use objc2::rc::autoreleasepool;
use objc2_app_kit::{NSScreen, NSWorkspace};
use objc2_foundation::{NSDictionary, NSString, NSURL};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{
    env::var,
    path::{Path, PathBuf},
    process::Command,
};

// --- OS specific code ---

/// Checks if the user's OS is currently in dark mode
///
/// # Errors
///
/// Returns a `MacOSError` with the `DarkModeError` variant if the command to determine
/// OS dark mode state cannot be executed. It can also return an error if the output
/// cannot be parsed.
pub fn is_dark_mode_active() -> Result<bool, MacOSError> {
    let output = Command::new("defaults")
        .arg("read")
        .arg("-g")
        .arg("AppleInterfaceStyle")
        .output()
        .map_err(|_| MacOSError::DarkModeError)?;
    let output_str = String::from_utf8(output.stdout).map_err(|_| MacOSError::DarkModeError)?;
    let dark_mode_enabled = matches!(output_str.trim().to_lowercase().as_str(), "dark");
    Ok(dark_mode_enabled)
}

/// Retrieves the resolution of the main display in pixels.
///
/// This function runs the `system_profiler` command with the `SPDisplaysDataType` and
/// `-detailLevel mini` arguments, then parses the output to extract the resolution of
/// the display marked as the main display. If no main display can be found or if the
/// resolution cannot be parsed from the output, this function returns an `Err`
/// containing a `MacOSError` with the `ResolutionNotFound` variant. If the
/// `system_profiler` command cannot be executed for any reason, this function will
/// return an `Err` containing a `MacOSError` with the `SystemProfilerError` variant.
///
/// # Errors
///
/// If the `system_profiler` command cannot be executed for any reason, this function
/// will return an `Err` containing a `MacOSError` with the `SystemProfilerError`
/// variant.
///
/// If the resolution of the main display cannot be found in the output of
/// the `system_profiler` command, this function will return an `Err` containing a
/// `MacOSError` with the `ResolutionNotFound` variant.
pub fn get_screen_resolution() -> Result<(u32, u32), MacOSError> {
    let output = Command::new("system_profiler")
        .arg("SPDisplaysDataType")
        .arg("-detailLevel")
        .arg("mini")
        .output()
        .map_err(|_| MacOSError::SystemProfilerError)?;

    let (width, height) = parse_output(&String::from_utf8_lossy(&output.stdout))?;
    Ok((width, height))
}

/// Updates the wallpaper on all connected displays to the image at the given path.
///
/// NOTE: as of v1.1.4
/// This implementation uses the native AppKit API (`NSWorkspace::setDesktopImageURL`)
/// rather than invoking `osascript`. Using the system API avoids several issues with
/// AppleScript-based approaches, including inconsistent behavior across multiple
/// displays and unintended changes to Mission Control wallpaper settings.
///
/// The provided `path` must reference a valid image file on the local filesystem.
/// The file should already exist and be fully written before this function is called,
/// otherwise macOS may continue displaying a cached version of the previous image.
///
/// The given path is set directly, matching the behavior on other platforms.
/// Previously (v1.1.4 through v1.3.1), macOS copied each render over one of two slot
/// files (`astra_1.png`/`astra_2.png`) because re-setting the same path could leave a
/// stale cached image on screen. That workaround is no longer needed: every render
/// already has a unique timestamped name, so macOS always sees a new path. The
/// System Settings → Wallpaper → "Your Photos" entries that unique names
/// accumulate are cleaned up separately by auto-clean (see
/// `prune_stale_wallpaper_gallery_entries`), which also restarts the wallpaper
/// agents so the gallery refreshes immediately.
///
/// # Platform
///
/// This function is only supported on macOS.
///
/// # Errors
///
/// - Returns `Err(MacOSError::OS(_))` if:
///   - The function is called from a non-main thread (required by AppKit).
///   - The path cannot be converted to a valid UTF-8 string.
///   - macOS fails to set the wallpaper for any display.
pub fn update_wallpaper(path: PathBuf) -> Result<(), MacOSError> {
    autoreleasepool(|_| {
        let mtm = MainThreadMarker::new()
            .ok_or_else(|| MacOSError::OS("update_wallpaper must run on the main thread".into()))?;

        let workspace = NSWorkspace::sharedWorkspace();
        let screens = NSScreen::screens(mtm);

        let path_str = path
            .to_str()
            .ok_or_else(|| MacOSError::OS("Invalid UTF-8 path".into()))?;
        let ns_string = NSString::from_str(path_str);
        let url = NSURL::fileURLWithPath(&ns_string);
        // Real update
        for screen in screens.iter() {
            unsafe {
                workspace
                    .setDesktopImageURL_forScreen_options_error(
                        &url,
                        screen.as_ref(),
                        &NSDictionary::new(),
                    )
                    .map_err(|e| MacOSError::OS(format!("Wallpaper update failed: {:?}", e)))?;
            }
        }

        Ok(())
    })
}

/// Path to the WallpaperAgent view-model cache for desktop image wallpapers.
///
/// This cache backs System Settings → Wallpaper → "Your Photos". It lives in the
/// per-user Darwin cache directory, which is a sibling of `TMPDIR` (`.../C/` next
/// to `.../T/`). Returns `None` if the cache does not exist yet.
fn wallpaper_gallery_path() -> Option<PathBuf> {
    let cache_root = std::env::temp_dir().parent()?.join("C");
    let path = cache_root.join(
        "com.apple.wallpaper.agent/com.apple.wallpaper.view-model-cache\
         /extension-com.apple.wallpaper.extension.image-desktop",
    );
    path.is_file().then_some(path)
}

/// Directory holding the per-entry thumbnail images referenced by the wallpaper
/// gallery cache and the extension's history (`copyURL`). Thumbnails are only ever
/// deleted from inside this directory. Returns `None` if the user's home directory
/// cannot be determined.
fn wallpaper_gallery_thumbnails_dir() -> Option<PathBuf> {
    UserDirs::new().map(|dirs| {
        dirs.home_dir()
            .join("Library/Containers/com.apple.wallpaper.extension.image/Data/Library/Caches")
    })
}

/// Path to the image wallpaper extension's preferences plist. It persists the
/// "Your Photos" image history that the System Settings gallery is rebuilt from.
/// Returns `None` if the file does not exist yet (the extension has never shown an
/// image on this machine).
fn wallpaper_gallery_history_path() -> Option<PathBuf> {
    UserDirs::new()
        .map(|dirs| {
            dirs.home_dir().join(
                "Library/Containers/com.apple.wallpaper.extension.image/Data/Library\
                 /Preferences/com.apple.wallpaper.extension.image.plist",
            )
        })
        .filter(|path| path.is_file())
}

/// Removes stale Astra entries from the macOS System Settings wallpaper gallery.
///
/// macOS keeps two copies of that history:
/// - WallpaperAgent's view-model cache, which renders System Settings → Wallpaper →
///   "Your Photos" (`prune_wallpaper_gallery_file`);
/// - the image wallpaper extension's `ChoiceRequests.ImageFiles` history, from
///   which WallpaperAgent rebuilds the gallery
///   (`prune_wallpaper_extension_image_files`).
///
/// Both must be pruned, and the wallpaper services must be stopped first: the image
/// extension keeps the full history in memory and writes it back when it exits, so
/// pruning while it runs would be undone by that final write. The services are
/// `SIGSTOP`ped while the files are rewritten and `SIGKILL`ed afterwards so that
/// in-memory history is discarded, and launchd relaunches them with the pruned files.
///
/// The services are only stopped when there is something to prune, so a clean that
/// has nothing to remove does not disturb the running wallpaper. An open System
/// Settings window can still re-add just-pruned entries from its own copy of the
/// history; the next clean (for example the next auto-clean) removes them again.
///
/// This never fails a clean: missing or unparsable files, or any I/O error, are
/// reported via verbose logging and skipped, because these formats are undocumented
/// and must never break wallpaper generation or deletion.
pub fn prune_stale_wallpaper_gallery_entries(config: &Config, wallpapers_dir: &Path) {
    let gallery_path = wallpaper_gallery_path();
    let history_path = wallpaper_gallery_history_path();
    if !crate::wallpaper_generators::wallpaper_gallery_has_stale_entries(
        gallery_path.as_deref(),
        history_path.as_deref(),
        wallpapers_dir,
    ) {
        return;
    }

    // Freeze the services while their files are rewritten, then kill them on drop so
    // launchd restarts them with the pruned state.
    let _frozen = FrozenWallpaperAgents::new(config);

    let thumbnails_dir = wallpaper_gallery_thumbnails_dir();
    if let Some(gallery_path) = &gallery_path {
        crate::wallpaper_generators::prune_wallpaper_gallery_file(
            config,
            gallery_path,
            wallpapers_dir,
            thumbnails_dir.as_deref(),
        );
    }
    if let Some(history_path) = &history_path {
        crate::wallpaper_generators::prune_wallpaper_extension_image_files(
            config,
            history_path,
            wallpapers_dir,
            thumbnails_dir.as_deref(),
        );
    }
}

/// Freezes the two macOS wallpaper services so they cannot restore the pruned
/// history, and kills them when dropped.
///
/// WallpaperImageExtension keeps the full image history in memory and writes it
/// back when it exits, so it must not be allowed to flush while the history is
/// pruned. `SIGSTOP` also keeps launchd from relaunching it mid-prune (a stopped
/// process still counts as running); `SIGKILL` then discards its in-memory state
/// without a flush, and launchd relaunches both services with the pruned files.
///
/// The kill happens in `Drop` so a panic while pruning still leaves working
/// wallpaper services behind.
struct FrozenWallpaperAgents;

impl FrozenWallpaperAgents {
    fn new(config: &Config) -> Self {
        config.print_if_verbose("Stopping macOS wallpaper services to prune the wallpaper gallery");
        if let Err(e) = Command::new("killall")
            .args(["-STOP", "WallpaperImageExtension", "WallpaperAgent"])
            .status()
        {
            config.print_if_verbose(
                format!(
                    "WARN: could not stop the macOS wallpaper services: {e}; the gallery will refresh after the next login"
                )
                .as_str(),
            );
        }
        Self
    }
}

impl Drop for FrozenWallpaperAgents {
    fn drop(&mut self) {
        // Let cfprefsd notice the rewritten preference files before the services
        // restart and read them.
        std::thread::sleep(std::time::Duration::from_millis(50));
        let _ = Command::new("killall")
            .args(["-9", "WallpaperImageExtension", "WallpaperAgent"])
            .status();
    }
}

/// Opens the given file in the user's default editor.
/// This function will first check the `EDITOR` environment variable, and if it is not set,
/// it will default to using the `open` command.
///
/// # Errors
/// - Returns a `MacOSError` with the `OpenEditorError` variant if the command to open the
///   file cannot be executed for any reason.
pub fn open_editor(config: &Config, path: PathBuf) -> Result<(), MacOSError> {
    let editor = var("EDITOR").unwrap_or("open".to_string());
    let _ = match editor.as_str() {
        "open" => {
            config.print_if_verbose("Using default editor");
            Command::new("open")
                .arg("-t")
                .arg(path)
                .spawn()
                .map_err(|_| MacOSError::OpenEditorError)?
                .wait()
                .map_err(|_| MacOSError::OpenEditorError)?
        }
        editor => {
            config.print_if_verbose(&format!("Using editor: {}", editor));
            Command::new(editor)
                .arg(path)
                .spawn()
                .map_err(|_| MacOSError::OpenEditorError)?
                .wait()
                .map_err(|_| MacOSError::OpenEditorError)?
        }
    };
    Ok(())
}

/// CRUD operator function for interfacing with the launchd system in macOS
///
/// This function will take in the configuration struct and check if the user
/// config contains a frequency key/value.
///
/// - If key/value is defined:
///   1. Check that frequency with launchctl is set with proper interval.
///      IF not, then install launchctl job and continue
///   2. Check if duration between current_timestamp and last execution of astra is greater than
///      frequency.
///      IF so, then proceed with program execution (returns true)
///      IF not, then program execution will stop and no wallpaper update (returns false)
/// - If key/value is not defined, remove the astra job from launchctl
///
/// The job is defined in the User Agents location (~/Library/LaunchAgents/)
pub fn handle_frequency(config: &Config) -> Result<bool, MacOSError> {
    if let Some(frequency) = config.frequency() {
        match launchctl_check_existence_of_astra_job()? {
            Some(interval) => {
                if interval != MAC_OS_LAUNCHCTL_INTERVAL {
                    launchctl_install_astra_freq()?;
                }
            }
            None => launchctl_install_astra_freq()?,
        }

        let current_timestamp_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| MacOSError::OS("time should go forward".to_string()))?
            .as_secs();
        if current_timestamp_secs - retrieve_last_execution_time()? < frequency.to_seconds() {
            return Ok(false);
        }
    } else {
        launchctl_uninstall_astra_freq()?;
    }
    Ok(true)
}

// --- OS specific code ---

// --- Helper functions ---

/// Helper function that retrieves the last execution time from the `last_exec.txt` file.
/// This time stamp can be set using the save_last_execution_time function below
fn retrieve_last_execution_time() -> Result<u64, MacOSError> {
    let proj_dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .ok_or_else(|| MacOSError::OS("could not derive data_dir".to_string()))?;
    let path_to_time_stamp_file = proj_dirs.data_dir().join("last_exec.txt");
    let timestamp = if path_to_time_stamp_file.exists() {
        fs::read_to_string(&path_to_time_stamp_file)
            .map_err(|e| MacOSError::OS(format!("failed to read last_exec.txt: {e}")))?
            .splitn(2, "=")
            .nth(1)
            .expect("should be a time stamp")
            .trim()
            .parse::<u64>()
            .map_err(|e| MacOSError::ParseError(e.to_string()))?
    } else {
        // IF no file is found, then assume first execution time was UNIX_EPOCH
        0
    };
    Ok(timestamp)
}

// TODO: use this func in main function loop if OS is mac and successful execution

/// Helper function that saves the last execution time when the `astra` command is ran in
/// data_directory. The file is named `last_exec.txt`.
///
/// This function is required for the handle_frequency function as macOS implementation
/// uses launchd job with 10 minute interval. This interval checks this time file
/// to determine if duration from last exec is higher than the frequency specified by user.
pub fn save_last_execution_time() -> Result<(), MacOSError> {
    let proj_dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .ok_or_else(|| MacOSError::OS("could not derive data_dir".to_string()))?;
    let path_to_time_stamp_file = proj_dirs.data_dir().join("last_exec.txt");
    let seconds_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| MacOSError::OS("time should go forward".to_string()))?
        .as_secs();
    fs::write(
        &path_to_time_stamp_file,
        format!("last_run = {seconds_timestamp}"),
    )
    .map_err(|e| MacOSError::OS(format!("failed to create/update last_exec.txt file: {e}")))?;
    Ok(())
}

/// Parses the output of the `system_profiler` command with the `SPDisplaysDataType`
/// and `-detailLevel mini` arguments.
///
/// This function first finds the line with `Main Display: Yes` and counts the number
/// of spaces preceding it until a newline. It then finds all lines with the same
/// number of spaces preceding/succeeding them and stores them in a vector. It then finds the
/// line with `Resolution:` and extracts the next two numbers from it, returning them
/// as a `(width, height)` tuple.
///
/// # Errors
///
/// If the line containing `Main Display: Yes` cannot be found in the output of the
/// `system_profiler` command, this function will return an `Err` containing a
/// `MacOSError` with the `MainDisplayNotFound` variant.
///
/// If the resolution of the main display cannot be found in the output of
/// the `system_profiler` command, this function will return an `Err` containing a
/// `MacOSError` with the `ResolutionNotFound` variant.
fn parse_output(output: &str) -> Result<(u32, u32), MacOSError> {
    // find line with Main Display: Yes
    let main_display_idx = output
        .lines()
        .position(|x| x.contains("Main Display: Yes"))
        .ok_or(MacOSError::MainDisplayNotFound)?;

    // count spaces preceding it until new line
    let main_display_line = output
        .lines()
        .nth(main_display_idx)
        .ok_or(MacOSError::MainDisplayNotFound)?;
    let num_spaces = preceding_spaces(main_display_line);

    // grab all lines with that many spaces preceding them
    let mut properties: Vec<&str> = vec![];
    // check up
    let mut i = main_display_idx
        .checked_sub(1)
        .ok_or(MacOSError::ResolutionNotFound)?;
    while i > 0 {
        let line = output.lines().nth(i).expect("Unable to get line");
        let added_property = get_key_value_pair_based_on_spaces(&mut properties, line, num_spaces);
        if !added_property {
            break;
        }
        i -= 1;
    }
    // check down
    i = main_display_idx + 1;
    while i < output.lines().count() {
        let line = output.lines().nth(i).expect("Unable to get line");
        let added_property = get_key_value_pair_based_on_spaces(&mut properties, line, num_spaces);
        if !added_property {
            break;
        }
        i += 1;
    }

    // find line with Resolution: and grab next 2 numbers
    Ok(properties
        .iter()
        .find(|x| x.contains("Resolution:"))
        .ok_or(MacOSError::ResolutionNotFound)
        .and_then(|x| {
            let resolution_vals = x
                .split(" x ")
                .map(|x| {
                    let num: String = x.chars().filter(|c| c.is_ascii_digit()).collect();
                    num.parse::<u32>()
                        .map_err(|_| MacOSError::ResolutionNotFound)
                })
                .collect::<Result<Vec<u32>, MacOSError>>()?;

            if resolution_vals.len() != 2 {
                return Err(MacOSError::ResolutionNotFound);
            }
            Ok((resolution_vals[0], resolution_vals[1]))
        }))?
}

/// Counts the number of spaces preceding the first non-space character in a line.
///
/// This function calculates the number of leading spaces in the provided string `line`
/// by trimming the leading spaces and subtracting the length of the trimmed string from
/// the original string length. It returns the count of these leading spaces.
///
/// # Arguments
///
/// * `line` - A string slice that represents the line to be analyzed.
///
/// # Returns
///
/// * `usize` - The number of leading spaces in the line.
fn preceding_spaces(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

/// Checks if the number of spaces preceding the first non-space character in `line`
/// matches the value of `num_spaces`. If it does, it adds `line` to `properties`
/// and returns `true`. If not, it returns `false`.
///
/// # Arguments
///
/// * `properties` - A mutable vector of string slices to add lines to if they
///   match the number of spaces.
/// * `line` - A string slice that represents the line to be analyzed.
/// * `num_spaces` - The number of spaces that should precede the first
///   non-space character in `line`.
///
/// # Returns
///
/// * `bool` - `true` if `line` was added to `properties`; `false` otherwise.
fn get_key_value_pair_based_on_spaces<'a>(
    properties: &mut Vec<&'a str>,
    line: &'a str,
    num_spaces: usize,
) -> bool {
    if preceding_spaces(line) != num_spaces {
        false
    } else {
        properties.push(line);
        true
    }
}

// --- Helper functions ---

// --- Tests ---

#[cfg(test)]
mod macos_tests {
    use super::*;

    #[test]
    fn it_parses_valid_output() {
        let output = r#"
            Graphics/Displays:

                Apple M1:

                  Chipset Model: Apple M1
                  Type: GPU
                  Bus: Built-In
                  Total Number of Cores: 8
                  Vendor: Apple (0x106b)
                  Metal Support: Metal 3
                  Displays:
                    Color LCD:
                      Display Type: Built-In Retina LCD
                      Resolution: 2560 x 1600 Retina
                      Main Display: Yes
                      Mirror: Off
                      Online: Yes
                      Automatically Adjust Brightness: Yes
                      Connection Type: Internal
                    LG HDR WFHD:
                      Resolution: 2560 x 1080 (UW-UXGA - Ultra Wide - Ultra Extended Graphics Array)
                      UI Looks like: 2560 x 1080 @ 75.00Hz
                      Mirror: Off
                      Online: Yes
                      Rotation: Supported
            "#;

        let (width, height) = super::parse_output(output).unwrap();
        assert_eq!(width, 2560);
        assert_eq!(height, 1600);
    }

    #[test]
    fn it_fails_to_parse_invalid_output() {
        let output = "";
        let res = super::parse_output(output);
        assert_eq!(res.err().unwrap(), super::MacOSError::MainDisplayNotFound);
    }

    #[test]
    fn it_fails_when_no_main_display() {
        let output = "Main Display: No";
        let res = super::parse_output(output);
        assert_eq!(res.err().unwrap(), super::MacOSError::MainDisplayNotFound);
    }

    #[test]
    fn it_fails_when_no_resolution_found() {
        let output = "Main Display: Yes";
        let res = super::parse_output(output);
        assert_eq!(res.err().unwrap(), super::MacOSError::ResolutionNotFound);
    }

    #[test]
    fn it_counts_leading_spaces_correctly() {
        assert_eq!(preceding_spaces("    four leading spaces"), 4);
        assert_eq!(preceding_spaces("no leading spaces"), 0);
        assert_eq!(preceding_spaces("  two leading spaces"), 2);
        assert_eq!(preceding_spaces("      six leading spaces"), 6);
    }

    #[test]
    fn it_handles_empty_string() {
        assert_eq!(preceding_spaces(""), 0);
    }

    #[test]
    fn it_handles_only_spaces() {
        assert_eq!(preceding_spaces("     "), 5);
    }
    #[test]
    fn it_adds_line_with_correct_spaces() {
        let mut properties = vec![];
        let line = "    Key: Value";
        let num_spaces = 4;
        let added = get_key_value_pair_based_on_spaces(&mut properties, line, num_spaces);
        assert!(added);
        assert_eq!(properties, vec!["    Key: Value"]);
    }

    #[test]
    fn it_does_not_add_line_with_incorrect_spaces() {
        let mut properties = vec![];
        let line = "  Key: Value";
        let num_spaces = 4;
        let added = get_key_value_pair_based_on_spaces(&mut properties, line, num_spaces);
        assert!(!added);
        assert!(properties.is_empty());
    }
}

// --- Tests ---
