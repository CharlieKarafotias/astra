use super::cli::Config;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};

pub(super) fn update(config: &Config) -> Result<(), UpdateError> {
    config.print_if_verbose("Updating Astra...");
    // TODO: In future, support binary downloads
    check_dependencies()?;
    check_version()?;
    download()?;
    build()?;
    replace_binary()?;
    Ok(())
}

// Check if computer has dependencies required to update (Must have rust, cargo, git)
fn check_dependencies() -> Result<(), UpdateError> {
    todo!("Check if computer has dependencies required to update (Must have rust, cargo, git)");
    Ok(())
}

// Compare current version with the latest version on GitHub
fn check_version() -> Result<(), UpdateError> {
    todo!("Compare current version with the latest version on github");
    Ok(())
}

// If current version is outdated, download the latest version into temp folder
fn download() -> Result<(), UpdateError> {
    todo!("Download the latest version into temp folder");
    Ok(())
}

// Run `cargo build --release` on the temp folder
fn build() -> Result<(), UpdateError> {
    todo!("Run `cargo build --release` on the temp folder");
    Ok(())
}

// Replace the old binary with the new binary
fn replace_binary() -> Result<(), UpdateError> {
    todo!("Replace the old binary with the new binary");
    Ok(())
}

#[derive(Debug, PartialEq)]
pub enum UpdateError {
    GithubError(String),
}

impl Display for UpdateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            UpdateError::GithubError(msg) => write!(f, "Github Error: {}", msg),
        }
    }
}

impl Error for UpdateError {}