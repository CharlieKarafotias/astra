use super::super::super::{Config, Frequency};
use super::super::LinuxOSError;
use directories::BaseDirs;
use std::{
    env::current_exe,
    fs::{self, create_dir_all},
    path::{Path, PathBuf},
    process::Command,
};

/// Generates a service file which is used alongside the timer file from gen_timer_file().
/// The service file is consumed by systemd in Linux to trigger automatic executions of the Astra
/// binary. For details on service files, see [Arch Linux page](https://wiki.archlinux.org/title/Systemd/Timers#Service_units)
fn gen_service_file() -> Result<String, LinuxOSError> {
    let curr_exe_path = current_exe().map_err(|e| LinuxOSError::ExecutablePath(e.to_string()))?;
    let file_contents = format!(
        "[Unit]
Description=Astra Wallpaper Updater

[Service]
Type=oneshot
ExecStart={}
",
        curr_exe_path.to_string_lossy().to_string()
    );
    Ok(file_contents)
}

/// Generates a timer file to be used alongside the service file from gen_service_file().
/// The timer file is based off the Frequency set by a user in their configuration file
/// For details on timer files, see [Arch Linux page](https://wiki.archlinux.org/title/Systemd/Timers#Timer_units)
fn gen_timer_file(f: Option<&Frequency>) -> Result<String, LinuxOSError> {
    if let Some(freq) = f {
        let file_content = format!(
            "[Unit]
Description=Run Astra Wallpaper on schedule

[Timer]
OnCalendar={}
Persistent=true

[Install]
WantedBy=timers.target
",
            freq.as_on_calendar_format()
        );
        Ok(file_content)
    } else {
        Err(LinuxOSError::GenerateTimer(
            "Frequency must be defined in user configuration to generate a timer file".to_string(),
        ))
    }
}

/// Installs Astra service and timer units to systemd
/// Steps:
///  1. Generate both unit and timer files
///  2. Create systemd/user folder in user config (if it doesn't exist)
///  3. Add unit and timer filers to new folder in systemd/user
///  4. Run systemctl command to reload daemon
///  5. Run systemctl command to enable astra timer
fn install_astra_service_and_timer(config: &Config) -> Result<(), LinuxOSError> {
    let systemd_dir = get_user_systemd_dir().ok_or_else(|| {
        LinuxOSError::PathNotFound("~/.config/systemd/user/ not found".to_string())
    })?;
    create_dir_all(&systemd_dir).map_err(|e| LinuxOSError::Os(e.to_string()))?;
    fs::write(systemd_dir.join("astra.service"), gen_service_file()?)
        .map_err(|e| LinuxOSError::Write(format!("astra.service file: {}", e.to_string())))?;
    fs::write(
        systemd_dir.join("astra.timer"),
        gen_timer_file(config.frequency())?,
    )
    .map_err(|e| LinuxOSError::Write(format!("astra.timer file: {}", e.to_string())))?;
    Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .output()
        .map_err(|e| {
            LinuxOSError::CommandError(format!("systemctl daemon-reload errored - {}", e))
        })?;
    Command::new("systemctl")
        .args(["--user", "enable", "--now", "astra.timer"])
        .output()
        .map_err(|e| {
            LinuxOSError::CommandError(format!("systemctl astra.timer enable errored - {}", e))
        })?;
    Ok(())
}

/// Uninstalls Astra service and timer units from systemd
/// Steps:
///  1. Run systemctl command to disable astra timer
///  2. Remove the service and timer files from systemd/user
///  3. Run systemctl command to reload daemon
fn uninstall_astra_serivice_and_timer() -> Result<(), LinuxOSError> {
    let systemd_dir = get_user_systemd_dir().ok_or_else(|| {
        LinuxOSError::PathNotFound("~/.config/systemd/user/ not found".to_string())
    })?;
    Command::new("systemctl")
        .args(["--user", "disable", "--now", "astra.timer"])
        .output()
        .map_err(|e| {
            LinuxOSError::CommandError(format!("systemctl astra.timer disable errored - {}", e))
        })?;
    if Path::new(systemd_dir.join("astra.timer").as_path()).exists() {
        fs::remove_file(systemd_dir.join("astra.timer"))
            .map_err(|e| LinuxOSError::Os(format!("failed to delete astra.timer - {}", e)))?;
    }
    if Path::new(systemd_dir.join("astra.service").as_path()).exists() {
        fs::remove_file(systemd_dir.join("astra.service"))
            .map_err(|e| LinuxOSError::Os(format!("failed to delete astra.service - {}", e)))?;
    }
    // Command::new("systemctl").args(["--user", "daemon-reload"]).status()?;
    Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .output()
        .map_err(|e| {
            LinuxOSError::CommandError(format!("systemctl daemon reload errored - {}", e))
        })?;
    Ok(())
}

fn get_user_systemd_dir() -> Option<PathBuf> {
    BaseDirs::new().map(|base| base.config_dir().join("systemd").join("user"))
}
