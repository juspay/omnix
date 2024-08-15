use crate::command::{CommandError, NixCmd};
use std::path::PathBuf;

/// Runs `nix copy` in Rust
pub async fn from_nix(
    cmd: &NixCmd,
    host: &str,
    extra_args: Vec<&PathBuf>,
) -> Result<(), CommandError> {
    let remote_address = format!("ssh://{}", host);
    let mut args = vec!["copy", "--to", &remote_address];

    // Convert PathBuf to String, then collect into a Vec<String>
    let extra_args_strings: Vec<String> = extra_args
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect();

    // Create a Vec<&str> from the String references
    let extra_args_str: Vec<&str> = extra_args_strings.iter().map(|s| s.as_str()).collect();

    // Now extend args with these &str slices
    args.extend(&extra_args_str);

    // Run the command with all arguments and capture the stdout
    cmd.run_with_args(&args).await?;

    Ok(())
}
