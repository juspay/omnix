//! Information about the user's Nix installation
use serde::{Deserialize, Serialize};
use std::fmt;
use tokio::sync::OnceCell;

use crate::{command::NixCmd, config::NixConfig, env::NixEnv, version::NixVersion};
use regex::Regex;

/// Type of Nix installation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NixInstallationType {
    /// Official Nix installation
    Official,
    /// Determinate Systems Nix
    DeterminateSystems,
}

impl fmt::Display for NixInstallationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NixInstallationType::Official => write!(f, "official"),
            NixInstallationType::DeterminateSystems => write!(f, "determinate-systems"),
        }
    }
}

impl NixInstallationType {
    /// Detect the installation type by examining `nix --version` output
    async fn detect() -> Result<Self, NixInfoError> {
        let cmd = NixCmd::default();
        let output = cmd
            .run_with_returning_stdout(&[], |cmd| {
                cmd.arg("--version");
            })
            .await
            .map_err(|e| NixInfoError::NixCmdError(crate::command::NixCmdError::CmdError(e)))?;
        let version_str = String::from_utf8_lossy(&output).trim().to_string();

        let patterns = [
            (
                r"^nix \(Determinate Nix [\d.]+\) (\d+)\.(\d+)\.(\d+)$",
                NixInstallationType::DeterminateSystems,
            ),
            (
                r"^nix \(Nix\) (\d+)\.(\d+)\.(\d+)$",
                NixInstallationType::Official,
            ),
            (r"^(\d+)\.(\d+)\.(\d+)$", NixInstallationType::Official),
        ];

        for (pattern, installation_type) in patterns {
            let re =
                Regex::new(pattern).map_err(|_| NixInfoError::InstallationTypeDetectionError)?;
            if re.is_match(&version_str) {
                return Ok(installation_type);
            }
        }

        // Default to Official if no pattern matches
        Ok(NixInstallationType::Official)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_installation_type_detection() {
        let test_cases = [
            ("nix (Nix) 2.28.4", NixInstallationType::Official),
            (
                "nix (Determinate Nix 3.8.5) 2.30.2",
                NixInstallationType::DeterminateSystems,
            ),
            ("2.28.4", NixInstallationType::Official),
        ];

        for (version_str, expected_type) in test_cases {
            let patterns = [
                (
                    r"^nix \(Determinate Nix [\d.]+\) (\d+)\.(\d+)\.(\d+)$",
                    NixInstallationType::DeterminateSystems,
                ),
                (
                    r"^nix \(Nix\) (\d+)\.(\d+)\.(\d+)$",
                    NixInstallationType::Official,
                ),
                (r"^(\d+)\.(\d+)\.(\d+)$", NixInstallationType::Official),
            ];

            let mut detected_type = NixInstallationType::Official; // default
            for (pattern, installation_type) in patterns {
                let re = Regex::new(pattern).unwrap();
                if re.is_match(version_str) {
                    detected_type = installation_type;
                    break;
                }
            }

            assert_eq!(
                detected_type, expected_type,
                "Failed for version string: '{}'",
                version_str
            );
        }
    }
}

/// All the information about the user's Nix installation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NixInfo {
    /// Nix version string
    pub nix_version: NixVersion,
    /// nix.conf configuration
    pub nix_config: NixConfig,
    /// Environment in which Nix was installed
    pub nix_env: NixEnv,
    /// Type of Nix installation
    pub installation_type: NixInstallationType,
}

static NIX_INFO: OnceCell<Result<NixInfo, NixInfoError>> = OnceCell::const_new();

impl NixInfo {
    /// Get the once version  of `NixInfo`
    pub async fn get() -> &'static Result<NixInfo, NixInfoError> {
        NIX_INFO
            .get_or_init(|| async {
                let nix_version = NixVersion::get().await.as_ref()?;
                let nix_config = NixConfig::get().await.as_ref()?;
                let info = NixInfo::new(*nix_version, nix_config.clone()).await?;
                Ok(info)
            })
            .await
    }

    /// Determine [NixInfo] on the user's system
    pub async fn new(
        nix_version: NixVersion,
        nix_config: NixConfig,
    ) -> Result<NixInfo, NixInfoError> {
        let nix_env = NixEnv::detect().await?;
        let installation_type = NixInstallationType::detect().await?;
        Ok(NixInfo {
            nix_version,
            nix_config,
            nix_env,
            installation_type,
        })
    }
}

/// Error type for [NixInfo]
#[derive(thiserror::Error, Debug)]
pub enum NixInfoError {
    /// A [crate::command::NixCmdError]
    #[error("Nix command error: {0}")]
    NixCmdError(#[from] crate::command::NixCmdError),

    /// A [crate::command::NixCmdError] with a static lifetime
    #[error("Nix command error: {0}")]
    NixCmdErrorStatic(#[from] &'static crate::command::NixCmdError),

    /// A [crate::env::NixEnvError]
    #[error("Nix environment error: {0}")]
    NixEnvError(#[from] crate::env::NixEnvError),

    /// A [crate::config::NixConfigError]
    #[error("Nix config error: {0}")]
    NixConfigError(#[from] &'static crate::config::NixConfigError),

    /// Installation type detection error
    #[error("Failed to detect installation type")]
    InstallationTypeDetectionError,
}
