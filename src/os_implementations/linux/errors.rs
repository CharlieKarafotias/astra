use std::error::Error;

#[derive(Debug, PartialEq)]
pub enum LinuxOSError {
    CommandError(String),
    DarkModeError(String),
    ExecutablePath(String),
    GenerateTimer(String),
    OpenEditorError,
    Os(String),
    ParseError(String),
    PathNotFound(String),
    ResolutionNotFound(String),
    Write(String),
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
            LinuxOSError::ExecutablePath(err_msg) => {
                write!(
                    f,
                    "Unable to determine path to current executable: {err_msg}"
                )
            }
            LinuxOSError::GenerateTimer(err_msg) => {
                write!(f, "Unable to generate time file: {err_msg}")
            }
            LinuxOSError::OpenEditorError => {
                write!(f, "Unable to open editor")
            }
            LinuxOSError::Os(err_msg) => {
                write!(f, "OS error: {err_msg}")
            }
            LinuxOSError::ParseError(err_msg) => {
                write!(f, "Unable to parse output: {err_msg}")
            }
            LinuxOSError::PathNotFound(err_msg) => {
                write!(f, "Unable to find path: {err_msg}")
            }
            LinuxOSError::ResolutionNotFound(err_msg) => {
                write!(
                    f,
                    "Unable to determine resolution of main display: {err_msg}"
                )
            }
            LinuxOSError::Write(err_msg) => {
                write!(f, "Unable to write file: {err_msg}")
            }
        }
    }
}

impl Error for LinuxOSError {}
