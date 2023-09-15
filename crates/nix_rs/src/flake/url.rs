//! Flake URL types
//!
//! See <https://nixos.org/manual/nix/stable/command-ref/new-cli/nix3-flake.html#url-like-syntax>
use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

/// A flake URL
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
            "github:juspay/nix-browser".into(),
            "github:nixos/nixpkgs".into(),
        ]
    }
}

impl Default for FlakeUrl {
    fn default() -> Self {
        // https://github.com/nammayatri/nammayatri/pull/2727
        "github:nammayatri/nammayatri/nix-meta".into()
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
