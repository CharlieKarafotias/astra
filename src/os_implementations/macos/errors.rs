use std::error::Error;

#[derive(Debug, PartialEq)]
pub enum MacOSError {
    DarkModeError,
    Launchctl(String),
    MainDisplayNotFound,
    OpenEditorError,
    OS(String),
    ResolutionNotFound,
    StringConversion,
    SystemProfilerError,
}

impl std::fmt::Display for MacOSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MacOSError::DarkModeError => write!(f, "Unable to determine dark mode status"),
            MacOSError::Launchctl(err_msg) => {
                write!(f, "Launchctl error: {err_msg}")
            }
            MacOSError::MainDisplayNotFound => write!(f, "Unable to determine main display"),
            MacOSError::OpenEditorError => write!(f, "Unable to open editor"),
            MacOSError::OS(err_msg) => write!(f, "General OS error: {err_msg}"),
            MacOSError::ResolutionNotFound => {
                write!(f, "Unable to determine resolution of main display")
            }
            MacOSError::StringConversion => write!(f, "Unable to convert to String"),
            MacOSError::SystemProfilerError => {
                write!(f, "Encountered error running system_profiler")
            }
        }
    }
}

impl Error for MacOSError {}
