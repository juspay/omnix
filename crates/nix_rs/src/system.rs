//! Information about the user's system
use os_info;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use std::fs;
use std::{env, io};
use thiserror::Error;

/// Information about the user's system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SysInfo {
    /// value of $USER
    pub current_user: String,
    /// OS information
    pub nix_system: NixSystem,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NixSystem {
    /// https://github.com/LnL7/nix-darwin
    NixDarwin,
    /// https://nixos.org/
    NixOS,
    /// Nix is individually installed on Linux or macOS
    Other(os_info::Type),
}

impl NixSystem {
    pub fn detect() -> Self {
        let os_type = os_info::get().os_type();
        match os_type {
            // To detect that we are on NixDarwin, we check if /etc/nix/nix.conf
            // is a symlink (which nix-darwin manages like NixOS does)
            os_info::Type::Macos if is_symlink("/etc/nix/nix.conf").unwrap_or(false) => {
                NixSystem::NixDarwin
            }
            os_info::Type::NixOS => NixSystem::NixOS,
            _ => NixSystem::Other(os_type),
        }
    }

    /// The Nix for this [NixSystem] is configured automatically through a `configuration.nix`
    pub fn has_configuration_nix(&self) -> bool {
        self == &NixSystem::NixOS || self == &NixSystem::NixDarwin
    }
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
    let metadata = fs::symlink_metadata(file_path)?;
    Ok(metadata.file_type().is_symlink())
}

impl SysInfo {
    /// Determine [SysInfo] on the user's system
    #[cfg(feature = "ssr")]
    pub async fn get_info() -> Result<SysInfo, SysInfoError> {
        let current_user = env::var("USER")?;
        let nix_system = NixSystem::detect();
        Ok(SysInfo {
            current_user,
            nix_system,
        })
    }
}
