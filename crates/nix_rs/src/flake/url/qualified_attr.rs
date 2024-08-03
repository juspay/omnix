use crate::{
    command::{NixCmd, NixCmdError},
    flake::eval::nix_eval_attr,
};

use super::FlakeUrl;

/// A qualified attribute that is expected to be found at the root of a flake
///
/// `RootQualifiedAttr { vec!["om.ci", "ci"] }` will locate the given attribute
/// under the one of these parent attributes, searched in that order.
pub struct RootQualifiedAttr {
    /// Candidate root attributes
    root_attrs: Vec<String>,
}

impl RootQualifiedAttr {
    pub fn new<T>(root_attrs: &[T]) -> Self
    where
        T: AsRef<str>,
    {
        let root_attrs = root_attrs.iter().map(|s| s.as_ref().to_string()).collect();
        Self { root_attrs }
    }

    /// Like [nix_eval_attr], but looks up the attribute in [FlakeUrl] under the
    /// current [RootQualifiedAttr]
    pub async fn eval_flake<T>(
        &self,
        cmd: &NixCmd,
        url: &FlakeUrl,
    ) -> Result<(T, FlakeUrl, Vec<String>), QualifiedAttrError>
    where
        T: Default + serde::de::DeserializeOwned,
    {
        nix_eval_qualified_attr(cmd, url, self.root_attrs.as_slice()).await
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

async fn nix_eval_qualified_attr<T>(
    cmd: &NixCmd,
    url: &FlakeUrl,
    root_attrs: &[String],
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
    for root_attr in root_attrs {
        let (url, attr) = flake_url.split_attr();
        let url = url.with_attr(format!("{}.{}", root_attr, attr.get_name()).as_str());
        if let Some(v) = nix_eval_attr(cmd, &url).await? {
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
