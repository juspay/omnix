//! Information about the environment in which Nix will run
use std::fmt::Display;

use os_info;
use serde::{Deserialize, Serialize};

/// The environment in which Nix operates
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NixEnv {
    /// value of $USER
    pub current_user: String,
    /// Underlying nix system information
    pub nix_system: NixSystem,
}

impl NixEnv {
    /// Determine [NixEnv] on the user's system
    #[cfg(feature = "ssr")]
    pub async fn detect() -> Result<NixEnv, NixEnvError> {
        let current_user = std::env::var("USER")?;
        let nix_system = NixSystem::detect().await;
        Ok(NixEnv {
            current_user,
            nix_system,
        })
    }
}

/// The system under which Nix is installed and operates
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NixSystem {
    /// https://github.com/LnL7/nix-darwin
    NixDarwin,
    /// https://nixos.org/
    NixOS,
    /// Nix is individually installed on Linux or macOS
    Other(os_info::Type),
}

impl Display for NixSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NixSystem::NixDarwin => write!(f, "nix-darwin"),
            NixSystem::NixOS => write!(f, "NixOS"),
            NixSystem::Other(os_type) => write!(f, "{}", os_type),
        }
    }
}

impl NixSystem {
    #[cfg(feature = "ssr")]
    pub async fn detect() -> Self {
        let os_info = tokio::task::spawn_blocking(os_info::get).await.unwrap();
        let os_type = os_info.os_type();
        async fn is_symlink(file_path: &str) -> std::io::Result<bool> {
            let metadata = tokio::fs::symlink_metadata(file_path).await?;
            Ok(metadata.file_type().is_symlink())
        }
        match os_type {
            // To detect that we are on NixDarwin, we check if /etc/nix/nix.conf
            // is a symlink (which nix-darwin manages like NixOS does)
            os_info::Type::Macos if is_symlink("/etc/nix/nix.conf").await.unwrap_or(false) => {
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

/// Errors while trying to fetch [NixEnv]
#[cfg(feature = "ssr")]
#[derive(thiserror::Error, Debug)]
pub enum NixEnvError {
    #[error("Failed to fetch ENV: {0}")]
    EnvVarError(#[from] std::env::VarError),
}
