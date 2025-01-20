//! Rust module for `nix copy`.
use crate::{
    command::{CommandError, NixCmd},
    store::uri::StoreURI,
};
use std::{ffi::OsStr, path::Path};

/// Options for `nix copy`.
#[derive(Debug, Clone, Default)]
pub struct NixCopyOptions {
    /// The URI of the store to copy from.
    pub from: Option<StoreURI>,
    /// The URI of the store to copy to.
    pub to: Option<StoreURI>,
    /// Do not check signatures.
    pub no_check_sigs: bool,
}

/// Copy store paths to a remote Nix store using `nix copy`.
///
/// # Arguments
///
/// * `cmd` - The `nix` command
/// * `host` - The remote host to copy to
/// * `paths` - The paths to copy. Limit this to be within the limit of Unix process arguments size limit.
pub async fn nix_copy<I, P>(
    cmd: &NixCmd,
    options: NixCopyOptions,
    paths: I,
) -> Result<(), CommandError>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path> + AsRef<OsStr>,
{
    cmd.run_with(|cmd| {
        cmd.arg("copy");
        if let Some(uri) = options.from {
            cmd.arg("--from").arg(uri.to_string());
        }
        if let Some(uri) = options.to {
            cmd.arg("--to").arg(uri.to_string());
        }
        if options.no_check_sigs {
            cmd.arg("--no-check-sigs");
        }
        cmd.args(paths);
    })
    .await?;
    Ok(())
}
