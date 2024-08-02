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

#[derive(thiserror::Error, Debug)]
pub enum QualifiedAttrError {
    #[error("Qualified attribute, {0}, not found in flake ref '{1}'")]
    MissingAttribute(String, FlakeUrl),

    #[error("Unexpected nested attribute: {0}")]
    UnexpectedNestedAttribute(String),

    #[error("Nix command error: {0}")]
    CommandError(#[from] NixCmdError),
}

// TODO: Move the following to attr.rs? along with
// with_fully_qualified_root_attr from url.rs

/// Like [nix_eval_attr], but tries multiple root attributes
///
/// Returns the value `T` along with the rest of attrs as a list
pub async fn nix_eval_qualified_attr<T>(
    cmd: &NixCmd,
    url: &FlakeUrl,
    root_attrs: &[&str],
) -> Result<(T, FlakeUrl, Vec<String>), QualifiedAttrError>
where
    T: Default + serde::de::DeserializeOwned,
{
    // Get 1st attr, retaining the rest
    let (flake_url, url_attr) = url.split_attr();
    let (flake_url, rest_attrs) = match url_attr.as_list().split_first() {
        None => (flake_url, vec![]),
        Some((name, rest)) => (flake_url.with_attr(name), rest.to_vec()),
    };

    // Try one of root_attrs
    for attr in root_attrs {
        let url = &flake_url.with_fully_qualified_root_attr(attr);
        if let Some(v) = nix_eval_attr(cmd, url).await? {
            return Ok((v, flake_url, rest_attrs));
        }
    }

    match url.split_attr().1.get_name().as_str() {
        "default" => Ok((Default::default(), flake_url, rest_attrs)),
        attr => {
            tracing::error!(
                "Qualified attr not found in flake ref '{}'. Expected one of: {}",
                url,
                root_attrs
                    .iter()
                    .map(|s| format!("{}.{}", s, attr))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            Err(QualifiedAttrError::MissingAttribute(
                attr.to_string(),
                url.clone(),
            ))
        }
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
