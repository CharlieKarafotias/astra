use std::error::Error;

#[derive(Debug, PartialEq)]
pub enum WindowsError {
    CommandError(String),
    DarkModeError(String),
    ExecutablePath(String),
    OpenEditorError(String),
    UpdateDesktopError(String),
}

impl std::fmt::Display for WindowsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowsError::CommandError(err) => {
                write!(f, "Unable to execute command: {err}")
            }
            WindowsError::DarkModeError(err) => {
                write!(f, "Unable to determine dark mode status: {err}")
            }
            WindowsError::ExecutablePath(err) => {
                write!(f, "Unable to determine path to current executable: {err}")
            }
            WindowsError::OpenEditorError(err) => {
                write!(f, "Unable to open file in default editor: {err}")
            }
            WindowsError::UpdateDesktopError(err) => {
                write!(f, "Unable to update desktop wallpaper: {err}")
            }
        }
    }
}
impl Error for WindowsError {}
