mod linux;
mod macos;
mod windows;

#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(target_os = "windows")]
pub use windows::*;
