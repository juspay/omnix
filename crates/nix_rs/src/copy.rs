use crate::{
    command::{CommandError, NixCmd},
    store::uri::StoreURI,
};
use std::path::Path;

/// Copy store paths to a remote Nix store using `nix copy`.
///
/// # Arguments
///
/// * `cmd` - The `nix` command
/// * `host` - The remote host to copy to
/// * `paths` - The (locally available) store paths to copy
pub async fn nix_copy(
    cmd: &NixCmd,
    store_uri: &StoreURI,
    paths: &[&Path],
) -> Result<(), CommandError> {
    let mut args = vec![
        "copy".to_string(),
        "--to".to_string(),
        store_uri.to_string(),
    ];
    for path in paths {
        args.push(path.to_string_lossy().into_owned());
    }
    cmd.run_with_args(&args).await
}
