//! Information about the user's Nix installation
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;

use crate::{
    config::NixConfig,
    env::NixEnv,
    version::{NixInstallationType, NixVersion},
};

/// All the information about the user's Nix installation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NixInfo {
    /// Nix version string
    pub nix_version: NixVersion,
    /// nix.conf configuration
    pub nix_config: NixConfig,
    /// Environment in which Nix was installed
    pub nix_env: NixEnv,
}

impl NixInfo {
    /// Get the installation type (derived from nix_version)
    pub fn installation_type(&self) -> NixInstallationType {
        self.nix_version.installation_type()
    }
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
        Ok(NixInfo {
            nix_version,
            nix_config,
            nix_env,
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
}
