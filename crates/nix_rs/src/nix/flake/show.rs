//! Rust module for `nix flake show`

use super::{outputs::FlakeOutputs, url::FlakeUrl};
use crate::nix::command::{NixCmd, NixCmdError, Refresh};

/// Run `nix flake show` on the given flake url
pub async fn run_nix_flake_show(
    flake_url: &FlakeUrl,
    refresh: Refresh,
) -> Result<FlakeOutputs, NixCmdError> {
    let v = NixCmd {
        refresh,
        ..NixCmd::default()
    }
    .run_with_args_expecting_json(&[
        "flake",
        "show",
        "--legacy", // for showing nixpkgs legacyPackages
        "--allow-import-from-derivation",
        "--json",
        &flake_url.to_string(),
    ])
    .await?;
    Ok(v)
}

#[tokio::test]
#[ignore] // Requires network, so won't work in Nix
async fn test_nix_flake_show() {
    let flake_url = "nixpkgs".into();
    assert!(run_nix_flake_show(&flake_url, false.into()).await.is_ok());
}
