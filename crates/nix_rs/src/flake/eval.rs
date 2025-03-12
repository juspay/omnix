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
    nix_eval_(nixcmd, opts, url, false).await
}

/// Like [nix_eval] but return `None` if the attribute is missing
pub async fn nix_eval_maybe<T>(
    cmd: &NixCmd,
    opts: &FlakeOptions,
    url: &FlakeUrl,
) -> Result<Option<T>, NixCmdError>
where
    T: Default + serde::de::DeserializeOwned,
{
    let result = nix_eval_(cmd, opts, url, true).await;
    match result {
        Ok(v) => Ok(Some(v)),
        Err(err) if error_is_missing_attribute(&err) => {
            Ok(None) // Attr is missing
        }
        Err(err) => Err(err),
    }
}

async fn nix_eval_<T>(
    nixcmd: &NixCmd,
    opts: &FlakeOptions,
    url: &FlakeUrl,
    capture_stderr: bool,
) -> Result<T, NixCmdError>
where
    T: serde::de::DeserializeOwned,
{
    let stdout = nixcmd
        .run_with(&["eval"], |cmd| {
            cmd.stdout(Stdio::piped());
            if capture_stderr {
                cmd.stderr(Stdio::piped());
            }
            cmd.args(["--json"]);
            opts.use_in_command(cmd);
            cmd.arg(url.to_string());
            // Avoid Nix from dumping logs related to `--override-input` use. Yes, this requires *double* use of `--quiet`.
            cmd.args(["--quiet", "--quiet"]);
        })
        .await?;
    let v = serde_json::from_slice::<T>(&stdout)?;
    Ok(v)
}

/// Check that [NixCmdError] is a missing attribute error
fn error_is_missing_attribute(err: &NixCmdError) -> bool {
    if let NixCmdError::CmdError(CommandError::ProcessFailed { stderr, .. }) = err {
        if stderr.contains("does not provide attribute") {
            return true;
        }
    }
    false
}
