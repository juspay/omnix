//! Rust module for `nix flake show`

use super::{outputs::FlakeOutputs, url::FlakeUrl};
use crate::nix::command::{NixCmd, Refresh};
use leptos::*;

/// Run `nix flake show` on the given flake url
pub async fn run_nix_flake_show(
    flake_url: &FlakeUrl,
    refresh: Refresh,
) -> Result<FlakeOutputs, ServerFnError> {
    let mut cmd = NixCmd {
        refresh,
        ..NixCmd::default()
    }
    .command();
    cmd.args(vec![
        "flake",
        "show",
        "--legacy", // for showing nixpkgs legacyPackages
        "--allow-import-from-derivation",
        "--json",
        &flake_url.to_string(),
    ]);
    let stdout: Vec<u8> = crate::command::run_command(&mut cmd).await?;
    let v = serde_json::from_slice::<FlakeOutputs>(&stdout)?;
    Ok(v)
}

#[tokio::test]
#[ignore] // Requires network, so won't work in Nix
async fn test_nix_flake_show() {
    let flake_url = "nixpkgs".into();
    assert!(run_nix_flake_show(&flake_url, false.into()).await.is_ok());
}
