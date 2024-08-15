use crate::command::{NixCmd, NixCmdError};

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::url::FlakeUrl;

#[derive(Serialize, Deserialize, Debug)]
pub struct FlakeMetadata {
    pub path: PathBuf,
}

/// Runs `nix flake metadata json` in Rust
pub async fn from_nix(cmd: &NixCmd, flake_url: &FlakeUrl) -> Result<FlakeMetadata, NixCmdError> {
    let json = cmd
        .run_with_args_expecting_json::<FlakeMetadata>(&[
            "flake",
            "metadata",
            "--json",
            &flake_url.0,
        ])
        .await?;
    Ok(json)
}
