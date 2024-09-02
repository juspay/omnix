use serde_json::Value;

use crate::{
    command::{NixCmd, NixCmdError},
    flake::{eval::nix_eval_attr, outputs::FlakeOutputs},
};

use super::FlakeUrl;

/// Like [nix_eval_attr] but relocates attr under one of  the given root attributes
///
/// A qualified attribute is expected to be found at the root of a flake
///
/// `&["om.ci", "ci"]` will locate the given attribute
/// under the one of these parent attributes, searched in that order.
pub async fn nix_eval_qualified_attr<T, S>(
    cmd: &NixCmd,
    url: &FlakeUrl,
    root_attrs: &[S],
) -> Result<(T, Vec<String>), QualifiedAttrError>
where
    S: AsRef<str>,
    T: Default + serde::de::DeserializeOwned,
{
    // Try one of root_attrs to see if it exists in flake
    for root_attr in root_attrs {
        let qualified_url = url.with_attr(root_attr.as_ref());
        if let Some(v) = nix_eval_attr(cmd, &qualified_url).await? {
            return Ok((v, url.get_attr().as_list()));
        }
    }

    // When none of root_attr matches, return default
    match url.get_attr().0 {
        None => Ok((Default::default(), vec![])),
        Some(attr) => Err(QualifiedAttrError::UnexpectedAttribute(attr)),
    }
}

/// Look for a qualified attribute in the flake output
///
/// A qualified attribute is expected to be found at the root of a flake
///
/// `&["om.ci", "ci"]` will locate the given attribute
/// under the one of these parent attributes, searched in that order.
pub async fn find_qualified_attr_in_flake_output<T, S>(
    cmd: &NixCmd,
    url: &FlakeUrl,
    root_attrs: &[S],
) -> Result<(T, Vec<String>), QualifiedAttrError>
where
    S: AsRef<str>,
    T: Default + serde::de::DeserializeOwned,
{
    let output = FlakeOutputs::from_nix(cmd, url).await?;
    let cfg = output.as_flattened_value()?;

    fn find_nested_value<'a>(config: &'a Value, search_path: &'a str) -> Option<&'a Value> {
        search_path
            .split('.')
            .try_fold(config, |acc, key| acc.get(key))
    }

    for root_attr in root_attrs {
        if let Some(v) = find_nested_value(&cfg, root_attr.as_ref()) {
            let v: T = serde_json::from_value(v.clone())?;
            return Ok((v, url.get_attr().as_list()));
        }
    }

    match url.get_attr().0 {
        None => Ok((Default::default(), vec![])),
        Some(attr) => Err(QualifiedAttrError::UnexpectedAttribute(attr)),
    }
}

#[derive(thiserror::Error, Debug)]
pub enum QualifiedAttrError {
    #[error("Unexpected attribute, when config not present in flake: {0}")]
    UnexpectedAttribute(String),

    #[error("Nix command error: {0}")]
    CommandError(#[from] NixCmdError),

    #[error("Serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
}
