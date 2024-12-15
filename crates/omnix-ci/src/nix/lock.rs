//! Functions for working with `nix flake lock`.

use anyhow::{Ok, Result};
use nix_rs::{
    command::NixCmd,
    flake::{self, command::FlakeOptions, url::FlakeUrl},
};

/// Make sure that the `flake.lock` file is in sync.
pub async fn nix_flake_lock_check(nixcmd: &NixCmd, url: &FlakeUrl) -> Result<()> {
    flake::command::lock(
        nixcmd,
        &FlakeOptions::default(),
        &["--no-update-lock-file"],
        url,
    )
    .await?;
    Ok(())
}
