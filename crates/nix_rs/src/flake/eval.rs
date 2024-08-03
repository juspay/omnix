use crate::command::{CommandError, NixCmd, NixCmdError};

use super::url::FlakeUrl;

/// Run `nix eval <url> --json` and parse its JSON
///
/// If the attribute is missing, return None.
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
