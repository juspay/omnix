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
    ///
    /// Returns the parsed value, first attribute ("default" if none) and the remaining attributes in the [FlakeUrl]
    pub async fn eval_flake<T>(
        &self,
        cmd: &NixCmd,
        url: &FlakeUrl,
    ) -> Result<(T, Vec<String>), QualifiedAttrError>
    where
        T: Default + serde::de::DeserializeOwned,
    {
        nix_eval_qualified_attr(cmd, url, self.root_attrs.as_slice()).await
    }
}

#[derive(thiserror::Error, Debug)]
pub enum QualifiedAttrError {
    #[error("Unexpected attribute, when config not present in flake: {0}")]
    UnexpectedAttribute(String),

    #[error("Nix command error: {0}")]
    CommandError(#[from] NixCmdError),
}

async fn nix_eval_qualified_attr<T>(
    cmd: &NixCmd,
    url: &FlakeUrl,
    root_attrs: &[String],
) -> Result<(T, Vec<String>), QualifiedAttrError>
where
    T: Default + serde::de::DeserializeOwned,
{
    // Try one of root_attrs to see if it exists in flake
    for root_attr in root_attrs {
        let qualified_url = url.with_attr(root_attr);
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
