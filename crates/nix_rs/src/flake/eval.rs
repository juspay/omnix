use crate::command::{NixCmd, NixCmdError};

use super::url::FlakeUrl;

/// Run `nix eval <url> --json` and parse its JSON
///
/// TODO: Is this even needed anymore?
pub async fn nix_eval_attr<T>(cmd: &NixCmd, url: &FlakeUrl) -> Result<T, NixCmdError>
where
    T: Default + serde::de::DeserializeOwned,
{
    cmd.run_with_args_expecting_json(&["eval", &url.0, "--json"])
        .await
}
