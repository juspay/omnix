use crate::command::{CommandError, NixCmd};

pub async fn run_nix_copy(
    cmd: &NixCmd,
    host: &str,
    omnix_input: &str,
    path: &str,
) -> Result<(), CommandError> {
    let remote_address = format!("ssh://{}", host);
    cmd.run_with_args(&["copy", "--to", &remote_address, omnix_input, path])
        .await?;
    Ok(())
}
