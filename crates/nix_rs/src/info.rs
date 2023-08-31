//! Information about the user's Nix installation
use serde::{Deserialize, Serialize};

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
