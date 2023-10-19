//! Information about the user's Nix installation
use serde::{Deserialize, Serialize};

use crate::{config::NixConfig, env::NixEnv, version::NixVersion};

/// All the information about the user's Nix installation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NixInfo {
    /// Nix version string
    pub nix_version: NixVersion,
    pub nix_config: NixConfig,
    pub nix_env: NixEnv,
}

impl NixInfo {
    /// Determine [NixInfo] on the user's system
    pub async fn from_nix(nix_cmd: &crate::command::NixCmd) -> Result<NixInfo, NixInfoError> {
        let nix_version = NixVersion::from_nix(nix_cmd).await?;
        let nix_config = NixConfig::from_nix(nix_cmd).await?;
        let nix_env = NixEnv::detect().await?;
        Ok(NixInfo {
            nix_version,
            nix_config,
            nix_env,
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum NixInfoError {
    #[error("Nix command error: {0}")]
    NixCmdError(#[from] crate::command::NixCmdError),

    #[error("Nix environment error: {0}")]
    NixEnvError(#[from] crate::env::NixEnvError),
}
