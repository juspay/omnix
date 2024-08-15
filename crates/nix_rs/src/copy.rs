use crate::command::{CommandError, NixCmd};
use std::path::PathBuf;

/// Runs `nix copy` in Rust
pub async fn from_nix(cmd: &NixCmd, host: &str, paths: Vec<&PathBuf>) -> Result<(), CommandError> {
    let remote_address = format!("ssh://{}", host);
    let mut args = vec!["copy", "--to", &remote_address];

    let paths_to_copy: Vec<&str> = paths
        .iter()
        .map(|path| path.as_path().to_str().unwrap_or_default())
        .collect();

    args.extend(&paths_to_copy);

    cmd.run_with_args(&args).await?;

    Ok(())
}
