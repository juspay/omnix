use crate::command::{NixCmd, NixCmdError};

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::url::FlakeUrl;

#[derive(Serialize, Deserialize, Debug)]
pub struct FlakeMetadata {
    pub path: PathBuf,
}

impl FlakeMetadata {
    /// Runs `nix flake metadata --json` for a given flake url in Rust
    pub async fn from_nix(
        cmd: &NixCmd,
        flake_url: &FlakeUrl,
    ) -> Result<FlakeMetadata, NixCmdError> {
        let metadata = cmd
            .run_with_args_expecting_json::<FlakeMetadata>(&[
                "flake",
                "metadata",
                "--json",
                &flake_url.0,
            ])
            .await?;
        Ok(metadata)
    }
}
