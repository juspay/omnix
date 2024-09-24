//! Work with `nix eval`
use tokio::io::AsyncReadExt;

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
    let mut stdout_handle = nixcmd
        .run_with_returning_stdout(|cmd| {
            cmd.args(["eval", "--json"]);
            opts.use_in_command(cmd);
            cmd.arg(url.to_string());
        })
        .await?;
    let mut stdout = Vec::new();

    stdout_handle
        .read_to_end(&mut stdout)
        .await
        .map_err(CommandError::ChildProcessError)?;
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
    cmd.run_with_args_expecting_json(&["eval", &url.0, "--json"])
        .await
}
