//! Work with `nix eval`
use std::process::Stdio;

use crate::command::{CommandError, NixCmd, NixCmdError};

use super::{command::FlakeOptions, url::FlakeUrl};

/// Run `nix eval <url> --json` and parse its JSON
pub async fn nix_eval<T>(
    nixcmd: &NixCmd,
    opts: &FlakeOptions,
    url: &FlakeUrl,
) -> Result<T, NixCmdError>
where
    T: serde::de::DeserializeOwned,
{
    let stdout = nixcmd
        .run_with(|cmd| {
            cmd.stdout(Stdio::piped());
            cmd.args(["eval", "--json"]);
            opts.use_in_command(cmd);
            cmd.arg(url.to_string());
            // Decrease logging verbosity to avoid polluting the `om` output
            // with logs other than the flake evaluation progress.
            cmd.args(["--quiet", "--quiet"]);
        })
        .await?;
    let v = serde_json::from_slice::<T>(&stdout)?;
    Ok(v)
}

/// Like [nix_eval] but takes an attribute to evaluate.
///
/// If the attribute is missing, return None.
///
/// TODO: Remove this function in favour of [nix_eval]
pub async fn nix_eval_attr<T>(cmd: &NixCmd, url: &FlakeUrl) -> Result<Option<T>, NixCmdError>
where
    T: Default + serde::de::DeserializeOwned,
{
    let result = cmd
        .run_with_args_expecting_json(&["eval", &url.0, "--json"])
        .await;
    match result {
        Ok(v) => Ok(Some(v)),
        Err(err) if error_is_missing_attribute(&err) => {
            Ok(None) // Attr is missing
        }
        Err(err) => Err(err),
    }
}

/// Check that [NixCmdError] is a missing attribute error
fn error_is_missing_attribute(err: &NixCmdError) -> bool {
    match err {
        NixCmdError::CmdError(CommandError::ProcessFailed { stderr, .. }) => {
            if let Some(stderr) = stderr {
                if stderr.contains("does not provide attribute") {
                    return true;
                }
            }
            false
        }
        _ => false,
    }
}
