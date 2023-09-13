//! Information about the user's Nix installation and system
use os_info;
use serde::{Deserialize, Serialize};
#[cfg(feature = "all")]
use std::env;

use crate::{config::NixConfig, version::NixVersion};

/// All the information about the user's Nix installation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NixInfo {
    /// Nix version string
    pub nix_version: NixVersion,
    pub nix_config: NixConfig,
}

impl NixInfo {
    /// Determine [NixInfo] on the user's system
    #[cfg(feature = "all")]
    pub async fn from_nix(
        nix_cmd: &crate::command::NixCmd,
    ) -> Result<NixInfo, crate::command::NixCmdError> {
        let nix_version = NixVersion::from_nix(nix_cmd).await?;
        let nix_config = NixConfig::from_nix(nix_cmd).await?;
        Ok(NixInfo {
            nix_version,
            nix_config,
        })
    }
}

/// Information about the user's system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SysInfo {
    /// value of $USER
    pub current_user: String,
    pub os: os_info::Type,
}

impl SysInfo {
    /// Determine [SysInfo] on the user's system
    #[cfg(feature = "all")]
    pub async fn get_info() -> Result<SysInfo, env::VarError> {
        let current_user = env::var("USER")?;
        let os = os_info::get().os_type();
        Ok(SysInfo { current_user, os })
    }
}
