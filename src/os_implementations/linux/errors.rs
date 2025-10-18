use std::error::Error;

#[derive(Debug, PartialEq)]
pub enum LinuxOSError {
    CommandError(String),
    DarkModeError(String),
    OpenEditorError,
    ParseError(String),
    ResolutionNotFound(String),
}

impl std::fmt::Display for LinuxOSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinuxOSError::CommandError(err_msg) => {
                write!(f, "Unable to execute command: {err_msg}")
            }
            LinuxOSError::DarkModeError(err_msg) => {
                write!(f, "Unable to determine dark mode status: {err_msg}")
            }
            LinuxOSError::OpenEditorError => {
                write!(f, "Unable to open editor")
            }
            LinuxOSError::ParseError(err_msg) => {
                write!(f, "Unable to parse output: {err_msg}")
            }
            LinuxOSError::ResolutionNotFound(err_msg) => {
                write!(
                    f,
                    "Unable to determine resolution of main display: {err_msg}"
                )
            }
        }
    }
}

impl Error for LinuxOSError {}
