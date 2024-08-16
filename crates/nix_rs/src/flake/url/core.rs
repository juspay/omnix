//! Flake URL types
//!
//! See <https://nixos.org/manual/nix/stable/command-ref/new-cli/nix3-flake.html#url-like-syntax>
use std::{
    fmt::{Display, Formatter},
    ops::Deref,
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

use super::attr::FlakeAttr;

/// A flake URL
///
/// See [syntax here](https://nixos.org/manual/nix/stable/command-ref/new-cli/nix3-flake.html#url-like-syntax).
///
/// Use `FromStr` to parse a string into a `FlakeUrl`. Or `From` or `Into` if
/// you know the URL is valid.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FlakeUrl(pub String);

impl AsRef<str> for FlakeUrl {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for FlakeUrl {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FlakeUrl {
    /// Provide real-world examples of flake URLs
    pub fn suggestions() -> Vec<FlakeUrl> {
        vec![
            str::parse("github:nammayatri/nammayatri").unwrap(),
            str::parse("github:srid/emanote").unwrap(),
            str::parse("github:srid/nixos-config").unwrap(),
            str::parse("github:juspay/nix-browser").unwrap(),
            str::parse("github:juspay/nix-dev-home").unwrap(),
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

    /// Split the [super::attr::FlakeAttr] out of the [FlakeUrl]
    pub fn split_attr(&self) -> (Self, FlakeAttr) {
        match self.0.split_once('#') {
            Some((url, attr)) => (FlakeUrl(url.to_string()), FlakeAttr(Some(attr.to_string()))),
            None => (self.clone(), FlakeAttr(None)),
        }
    }

    /// Return the [super::attr::FlakeAttr] of the [FlakeUrl]
    pub fn get_attr(&self) -> FlakeAttr {
        self.split_attr().1
    }

    /// Return the flake URL without the attribute
    pub fn without_attr(&self) -> Self {
        let (url, _) = self.split_attr();
        url
    }

    /// Return the flake URL with the given attribute
    pub fn with_attr(&self, attr: &str) -> Self {
        let (url, _) = self.split_attr();
        FlakeUrl(format!("{}#{}", url.0, attr))
    }

    /// Return the flake URL pointing to the sub-flake
    pub fn sub_flake_url(&self, dir: String) -> FlakeUrl {
        if dir == "." {
            self.clone()
        } else if let Some(path) = self.as_local_path() {
            // Local path; just join the dir
            let path_with_dir = path.join(dir);
            FlakeUrl(format!("path:{}", path_with_dir.display()))
        } else {
            // Non-path URL; append `dir` query parameter
            let mut url = self.0.clone();
            if url.contains('?') {
                url.push_str("&dir=");
            } else {
                url.push_str("?dir=");
            }
            url.push_str(&dir);
            FlakeUrl(url)
        }
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
            Ok(FlakeUrl(s.to_string()))
        }
    }
}

impl Display for FlakeUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flake_url_and_attr() {
        let url = FlakeUrl("github:srid/nixci".to_string());
        assert_eq!(url.split_attr(), (url.clone(), FlakeAttr(None)));
        assert_eq!(url.split_attr().1.as_list(), [] as [&str; 0]);

        let url = FlakeUrl("github:srid/nixci#extra-tests".to_string());
        assert_eq!(
            url.split_attr(),
            (
                FlakeUrl("github:srid/nixci".to_string()),
                FlakeAttr(Some("extra-tests".to_string()))
            )
        );
        assert_eq!(
            url.split_attr().1.as_list(),
            vec!["extra-tests".to_string()]
        );

        let url = FlakeUrl(".#foo.bar.qux".to_string());
        assert_eq!(
            url.split_attr(),
            (
                FlakeUrl(".".to_string()),
                FlakeAttr(Some("foo.bar.qux".to_string()))
            )
        );
        assert_eq!(
            url.split_attr().1.as_list(),
            vec!["foo".to_string(), "bar".to_string(), "qux".to_string()]
        )
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
        // Path refs
        let url = FlakeUrl(".".to_string());
        assert_eq!(url.sub_flake_url(".".to_string()), url.clone());
        assert_eq!(
            url.sub_flake_url("sub".to_string()),
            FlakeUrl("path:./sub".to_string())
        );

        // URI refs
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

    #[test]
    fn test_with_attr() {
        let url = FlakeUrl("github:srid/nixci".to_string());
        assert_eq!(
            url.with_attr("foo"),
            FlakeUrl("github:srid/nixci#foo".to_string())
        );

        let url: FlakeUrl = "github:srid/nixci#foo".parse().unwrap();
        assert_eq!(
            url.with_attr("bar"),
            FlakeUrl("github:srid/nixci#bar".to_string())
        );
    }
}
