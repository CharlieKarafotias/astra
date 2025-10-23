// NOTE: when developing for other targets, uncomment the line below with the debug_assertions
// to get compiler checks (or code on native system). Ensure to recomment before pushing.

// #[cfg(any(target_os = "macos", debug_assertions))]
#[cfg(target_os = "macos")]
mod macos;
// #[cfg(any(target_os = "macos", debug_assertions))]
#[cfg(target_os = "macos")]
pub use macos::*;

// #[cfg(any(target_os = "windows", debug_assertions))]
#[cfg(target_os = "windows")]
mod windows;
// #[cfg(any(target_os = "windows", debug_assertions))]
#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(any(target_os = "linux", debug_assertions))]
// #[cfg(target_os = "linux")]
mod linux;
#[cfg(any(target_os = "linux", debug_assertions))]
// #[cfg(target_os = "linux")]
pub use linux::*;
