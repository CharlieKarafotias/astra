use super::super::LinuxOSError;
use std::env::current_exe;

// INFO: use [this page](https://wiki.archlinux.org/title/Systemd/User)
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

fn gen_timer_file() -> Result<String, LinuxOSError> {
    todo!("impl me - needs frequency")
}

fn install_astra_service_and_timer() -> Result<(), LinuxOSError> {
    // NOTE: to reload + enable
    // Command::new("systemctl").args(["--user", "daemon-reload"]).status()?;
    // Command::new("systemctl".args(["--user", "enable", "--now", "astra.timer"]).status()?;
    todo!(
        "Implement me - will install in dirs::config_dir().join('systemd/user'); service called astra.service and timer called astra.timer"
    )
}

fn uninstall_astra_serivice_and_timer() -> Result<(), LinuxOSError> {
    // NOTE: to disable + remove + reload
    // Command::new("systemctl").args(["--user", "disable", "--now", "astra.timer"]).status().ok();
    // Remove service and timer files from systemd/user
    // Command::new("systemctl").args(["--user", "daemon-reload"]).status()?;
    todo!("impl uninstall")
}
