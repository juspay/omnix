use crate::command::{CommandError, NixCmd};
use std::path::PathBuf;

/// Copy store paths to a remote Nix store using `nix copy`.
///
/// # Arguments
///
/// * `cmd` - The `nix` command
/// * `host` - The remote host to copy to
/// * `paths` - The (locally available) store paths to copy
pub async fn nix_copy(
    cmd: &NixCmd,
    store_uri: &str,
    paths: &[PathBuf],
) -> Result<(), CommandError> {
    let mut args = vec!["copy".to_string(), "--to".to_string(), store_uri.to_owned()];
    for path in paths {
        args.push(path.to_string_lossy().into_owned());
    }
    cmd.run_with_args(&args).await
}
