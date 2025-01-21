//! Rust module for Nix flakes

pub mod command;
pub mod eval;
pub mod functions;
pub mod outputs;
pub mod schema;
pub mod system;
pub mod url;

use schema::FlakeSchemas;
use serde::{Deserialize, Serialize};

use system::System;
use tracing::instrument;

use self::{outputs::FlakeOutputs, url::FlakeUrl};

use crate::{
    command::{NixCmd, NixCmdError},
    config::NixConfig,
};

/// All the information about a Nix flake
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flake {
    /// The flake url which this struct represents
    pub url: FlakeUrl,
    /// Flake outputs derived from [FlakeSchemas]
    pub output: FlakeOutputs,
    // TODO: Add `nix flake metadata` info.
}

impl Flake {
    /// Get [Flake] info for the given flake url

    #[instrument(name = "flake", skip(nix_cmd))]
    pub async fn from_nix(
        nix_cmd: &NixCmd,
        nix_config: &NixConfig,
        url: FlakeUrl,
    ) -> Result<Flake, NixCmdError> {
        let schemas = FlakeSchemas::from_nix(nix_cmd, &url, &nix_config.system.value).await?;
        Ok(Flake {
            url,
            output: schemas.into(),
        })
    }
}
