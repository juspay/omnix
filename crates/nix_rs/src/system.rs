//! Information about the user's system
use os_info;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::{env, io};
#[cfg(feature = "ssr")]
use std::fs;

/// Information about the user's system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SysInfo {
    /// value of $USER
    pub current_user: String,
    /// Name of the OS
    pub os: os_info::Type,
    /// True only when the os is MacOS and `nix.conf` is a symlink
    pub uses_nix_darwin: bool,
}

/// Errors while trying to fetch system info
#[derive(Error, Debug)]
pub enum SysInfoError {
    #[error("Failed to read the file: {0}")]
    IOError(#[from] io::Error),

    #[error("Failed to fetch ENV: {0}")]
    EnvVarError(#[from] env::VarError),
}

#[cfg(feature = "ssr")]
fn is_symlink(file_path: &str) -> io::Result<bool> {
    let metadata = fs::metadata(file_path)?;
    Ok(metadata.file_type().is_symlink())
}

impl SysInfo {
    /// Determine [SysInfo] on the user's system
    #[cfg(feature = "ssr")]
    pub async fn get_info() -> Result<SysInfo, SysInfoError> {
        let current_user = env::var("USER")?;
        let os = os_info::get().os_type();
        let file_path = "/etc/nix/nix.conf";
        let uses_nix_darwin = (os == os_info::Type::Macos) && is_symlink(file_path)?;
        Ok(SysInfo { current_user, os , uses_nix_darwin })
    }
}
