//! Work with `nix flake metadata`
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::url::FlakeUrl;
use crate::command::{NixCmd, NixCmdError};

/// Flake metadata
///
/// See [Nix doc](https://nix.dev/manual/nix/2.18/command-ref/new-cli/nix3-flake-metadata)
#[derive(Serialize, Deserialize, Debug)]
pub struct FlakeMetadata {
    /// Original flake URL
    #[serde(rename = "originalUrl")]
    pub original_url: FlakeUrl,

    /// Locally cached path for the flake.
    pub path: PathBuf,
}

impl FlakeMetadata {
    /// Runs `nix flake metadata --json` for a given flake url
    pub async fn from_nix(
        cmd: &NixCmd,
        flake_url: &FlakeUrl,
    ) -> Result<FlakeMetadata, NixCmdError> {
        cmd.run_with_args_expecting_json::<FlakeMetadata>(&[
            "flake", "metadata", "--json", flake_url,
        ])
        .await
    }
}
