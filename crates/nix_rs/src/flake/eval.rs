use crate::command::{CommandError, NixCmd, NixCmdError};

use super::url::FlakeUrl;

/// Run `nix eval <url> --json` and parse its JSON
///
/// If the flake does not output the given attribute, return the [Default]
/// value of `T`.
pub async fn nix_eval_attr_json<T>(
    url: &FlakeUrl,
    default_if_missing: bool,
) -> Result<T, NixCmdError>
where
    T: Default + serde::de::DeserializeOwned,
{
    let nix = NixCmd::default();
    let result = nix
        .run_with_args_expecting_json(&["eval", url.0.as_str(), "--json"])
        .await;
    match result {
        Err(err) if default_if_missing && error_is_missing_attribute(&err) => {
            // The 'nixci' flake output attr is missing. User wants the default config.
            Ok(T::default())
        }
        r => r,
    }
}

/// Check that [NixCmdError] is a missing attribute error
fn error_is_missing_attribute(err: &NixCmdError) -> bool {
    match err {
        NixCmdError::CmdError(CommandError::ProcessFailed { stderr, .. }) => {
            stderr.contains("does not provide attribute")
        }
        _ => false,
    }
}
