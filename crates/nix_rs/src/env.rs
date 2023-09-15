//! Information about the environment in which Nix will run
use std::fmt::Display;

use os_info;
use serde::{Deserialize, Serialize};

/// The environment in which Nix operates
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NixEnv {
    /// value of $USER
    pub current_user: String,
    /// Underlying OS in which Nix runs
    pub os: OS,
}

impl NixEnv {
    /// Determine [NixEnv] on the user's system
    #[cfg(feature = "ssr")]
    pub async fn detect() -> Result<NixEnv, NixEnvError> {
        let current_user = std::env::var("USER")?;
        let os = OS::detect().await;
        Ok(NixEnv { current_user, os })
    }
}

/// The system under which Nix is installed and operates
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OS {
    /// On macOS
    MacOS {
        /// Using https://github.com/LnL7/nix-darwin
        nix_darwin: bool,
        arch: MacOSArch,
    },
    /// https://nixos.org/
    NixOS,
    /// Nix is individually installed on Linux or macOS
    Other(os_info::Type),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MacOSArch {
    Arm64(AppleEmulation),
    Other(Option<String>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppleEmulation {
    None,
    Rosetta,
}

impl AppleEmulation {
    #[cfg(feature = "ssr")]
    pub fn new() -> Self {
        use is_proc_translated::is_proc_translated;
        if is_proc_translated() {
            AppleEmulation::Rosetta
        } else {
            AppleEmulation::None
        }
    }
}

#[cfg(feature = "ssr")]
impl Default for AppleEmulation {
    fn default() -> Self {
        Self::new()
    }
}

impl MacOSArch {
    #[cfg(feature = "ssr")]
    pub fn from(os_arch: Option<&str>) -> MacOSArch {
        match os_arch {
            Some("arm64") => MacOSArch::Arm64(AppleEmulation::new()),
            other => MacOSArch::Other(other.map(|s| s.to_string())),
        }
    }
}

impl Display for OS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OS::MacOS {
                nix_darwin,
                arch: _,
            } => {
                if *nix_darwin {
                    write!(f, "nix-darwin")
                } else {
                    write!(f, "macOS")
                }
            }
            OS::NixOS => write!(f, "NixOS"),
            OS::Other(os_type) => write!(f, "{}", os_type),
        }
    }
}

impl OS {
    #[cfg(feature = "ssr")]
    pub async fn detect() -> Self {
        let os_info = tokio::task::spawn_blocking(os_info::get).await.unwrap();
        let os_type = os_info.os_type();
        let arch = MacOSArch::from(os_info.architecture());
        async fn is_symlink(file_path: &str) -> std::io::Result<bool> {
            let metadata = tokio::fs::symlink_metadata(file_path).await?;
            Ok(metadata.file_type().is_symlink())
        }
        match os_type {
            os_info::Type::Macos => {
                // To detect that we are on NixDarwin, we check if /etc/nix/nix.conf
                // is a symlink (which nix-darwin manages like NixOS does)
                let nix_darwin = is_symlink("/etc/nix/nix.conf").await.unwrap_or(false);
                OS::MacOS { nix_darwin, arch }
            }
            os_info::Type::NixOS => OS::NixOS,
            _ => OS::Other(os_type),
        }
    }

    /// The Nix for this [OS] is configured automatically through a `configuration.nix`
    pub fn has_configuration_nix(&self) -> bool {
        match self {
            OS::MacOS {
                nix_darwin,
                arch: _,
            } if *nix_darwin => true,
            OS::NixOS => true,
            _ => false,
        }
    }
}

/// Errors while trying to fetch [NixEnv]
#[cfg(feature = "ssr")]
#[derive(thiserror::Error, Debug)]
pub enum NixEnvError {
    #[error("Failed to fetch ENV: {0}")]
    EnvVarError(#[from] std::env::VarError),
}
