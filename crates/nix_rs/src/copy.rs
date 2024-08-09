use crate::command::{CommandError, NixCmd};
use anyhow::Result;

pub async fn nix_copy(
    cmd: &NixCmd,
    remote_address: &str,
    omnix_input: &str,
    path: &str,
) -> Result<(), CommandError> {
    cmd.run_with_args(&["copy", "--to", &remote_address, omnix_input, &path])
        .await;
    Ok(())
}
