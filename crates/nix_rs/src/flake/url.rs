//! Flake URL types
//!
//! See <https://nixos.org/manual/nix/stable/command-ref/new-cli/nix3-flake.html#url-like-syntax>
use std::{
    fmt::{Display, Formatter},
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

/// A flake URL
///
/// See [syntax here](https://nixos.org/manual/nix/stable/command-ref/new-cli/nix3-flake.html#url-like-syntax).
///
/// Use `FromStr` to parse a string into a `FlakeUrl`. Or `From` or `Into` if
/// you know the URL is valid.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FlakeUrl(pub String);

impl FlakeUrl {
    /// Provide real-world examples of flake URLs
    pub fn suggestions() -> Vec<FlakeUrl> {
        vec![
            FlakeUrl::default(),
            "github:srid/emanote".into(),
            "github:srid/nixos-config".into(),
            "github:juspay/nix-browser".into(),
            "github:juspay/nix-dev-home".into(),
            // Commented out until we figure out rendering performance and/or
            // search filtering/limit.
            // "github:nixos/nixpkgs".into(),
        ]
    }

    /// Return the local path if the flake URL is a local path
    ///
    /// Applicable only if the flake URL uses the [Path-like
    /// syntax](https://nixos.org/manual/nix/stable/command-ref/new-cli/nix3-flake.html#path-like-syntax)
    pub fn as_local_path(&self) -> Option<&Path> {
        let s = self.0.strip_prefix("path:").unwrap_or(&self.0);
        if s.starts_with('.') || s.starts_with('/') {
            // Strip query (`?..`) and attrs (`#..`)
            let s = s.split('?').next().unwrap_or(s);
            let s = s.split('#').next().unwrap_or(s);
            Some(Path::new(s))
        } else {
            None
        }
    }

    /// Return `<url>#<root-attr>.default` if `<url>` is passed
    ///
    /// If `<url>#foo` is passed, return `<url>#<root-attr>.foo`.
    pub fn with_fully_qualified_root_attr(&self, root_attr: &str) -> Self {
        let (url, attr) = self.split_attr();
        let name = attr.get_name();
        FlakeUrl(format!("{}#{}.{}", url.0, root_attr, name))
    }

    /// Split the [FlakeAttr] out of the [FlakeUrl]
    pub fn split_attr(&self) -> (Self, FlakeAttr) {
        match self.0.split_once('#') {
            Some((url, attr)) => (FlakeUrl(url.to_string()), FlakeAttr(Some(attr.to_string()))),
            None => (self.clone(), FlakeAttr(None)),
        }
    }

    /// Return the flake URL pointing to the sub-flake
    pub fn sub_flake_url(&self, dir: String) -> FlakeUrl {
        if dir == "." {
            self.clone()
        } else {
            let sep = if self.0.contains('?') { '&' } else { '?' };
            FlakeUrl(format!("{}{}dir={}", self.0, sep, dir))
        }
    }
}

impl Default for FlakeUrl {
    fn default() -> Self {
        "github:nammayatri/nammayatri".into()
    }
}

impl From<&str> for FlakeUrl {
    fn from(url: &str) -> Self {
        url.to_string().into()
    }
}

impl From<String> for FlakeUrl {
    fn from(url: String) -> Self {
        Self(url)
    }
}

impl From<PathBuf> for FlakeUrl {
    fn from(path: PathBuf) -> Self {
        FlakeUrl(format!("path:{}", path.display()))
    }
}

impl FromStr for FlakeUrl {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            Err("Empty string is not a valid Flake URL".to_string())
        } else {
            Ok(s.into())
        }
    }
}

impl Display for FlakeUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The attribute output part of a [FlakeUrl]
///
/// Example: `foo` in `.#foo`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FlakeAttr(Option<String>);

impl FlakeAttr {
    /// Get the attribute name.
    ///
    /// If attribute exists, then return "default".
    pub fn get_name(&self) -> String {
        self.0.clone().unwrap_or_else(|| "default".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flake_url() {
        let url = FlakeUrl("github:srid/nixci".to_string());
        assert_eq!(url.split_attr(), (url.clone(), FlakeAttr(None)));

        let url = FlakeUrl("github:srid/nixci#extra-tests".to_string());
        assert_eq!(
            url.split_attr(),
            (
                FlakeUrl("github:srid/nixci".to_string()),
                FlakeAttr(Some("extra-tests".to_string()))
            )
        );
    }

    #[test]
    fn test_as_local_flake() {
        let url = FlakeUrl("github:srid/nixci".to_string());
        assert_eq!(url.as_local_path(), None);

        let url = FlakeUrl(".".to_string());
        assert_eq!(url.as_local_path().map(|p| p.to_str().unwrap()), Some("."));

        let url = FlakeUrl("/foo".to_string());
        assert_eq!(url.as_local_path(), Some(std::path::Path::new("/foo")));

        let url = FlakeUrl("./foo?q=bar".to_string());
        assert_eq!(url.as_local_path(), Some(std::path::Path::new("./foo")));

        let url = FlakeUrl("./foo#attr".to_string());
        assert_eq!(url.as_local_path(), Some(std::path::Path::new("./foo")));

        let url = FlakeUrl("/foo?q=bar#attr".to_string());
        assert_eq!(url.as_local_path(), Some(std::path::Path::new("/foo")));

        let url = FlakeUrl("path:.".to_string());
        assert_eq!(url.as_local_path(), Some(std::path::Path::new(".")));

        let url = FlakeUrl("path:./foo".to_string());
        assert_eq!(url.as_local_path(), Some(std::path::Path::new("./foo")));

        let url = FlakeUrl("path:./foo?q=bar".to_string());
        assert_eq!(url.as_local_path(), Some(std::path::Path::new("./foo")));

        let url = FlakeUrl("path:./foo#attr".to_string());
        assert_eq!(url.as_local_path(), Some(std::path::Path::new("./foo")));

        let url = FlakeUrl("path:/foo?q=bar#attr".to_string());
        assert_eq!(url.as_local_path(), Some(std::path::Path::new("/foo")));
    }

    #[test]
    fn test_sub_flake_url() {
        let url = FlakeUrl("github:srid/nixci".to_string());
        assert_eq!(url.sub_flake_url(".".to_string()), url.clone());
        assert_eq!(
            url.sub_flake_url("dev".to_string()),
            FlakeUrl("github:srid/nixci?dir=dev".to_string())
        );
    }

    #[test]
    fn test_sub_flake_url_with_query() {
        let url = FlakeUrl("git+https://example.org/my/repo?ref=master".to_string());
        assert_eq!(url.sub_flake_url(".".to_string()), url.clone());
        assert_eq!(
            url.sub_flake_url("dev".to_string()),
            FlakeUrl("git+https://example.org/my/repo?ref=master&dir=dev".to_string())
        );
    }
}
