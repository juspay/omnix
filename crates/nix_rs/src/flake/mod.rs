//! Rust module for Nix flakes
pub mod outputs;
pub mod schema;
pub mod system;
pub mod url;

use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use tracing::instrument;

use self::{outputs::FlakeOutputs, schema::FlakeSchema, system::System, url::FlakeUrl};
#[cfg(feature = "ssr")]
use crate::command::NixCmdError;

/// All the information about a Nix flake
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flake {
    /// The flake url which this struct represents
    pub url: FlakeUrl,
    /// `nix flake show` output
    pub output: FlakeOutputs,
    /// Flake output schema (typed version of [FlakeOutputs])
    pub schema: FlakeSchema,
    // TODO: Add `nix flake metadata` info.
}

impl Flake {
    /// Get [Flake] info for the given flake url
    #[cfg(feature = "ssr")]
    #[instrument(name = "flake")]
    pub async fn from_nix(
        nix_cmd: &crate::command::NixCmd,
        url: FlakeUrl,
    ) -> Result<Flake, NixCmdError> {
        use crate::config::NixConfig;

        // TODO: Can we cache this?
        let nix_config = NixConfig::from_nix(nix_cmd).await?;
        let system = nix_config.system.value;
        let output = FlakeOutputs::from_nix(nix_cmd, &url).await?;
        let schema = FlakeSchema::from(&output, &system);
        Ok(Flake {
            url,
            output: output.clone(),
            schema,
        })
    }
}
