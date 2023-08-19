//! Rust module for `nix --version`
use leptos::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use thiserror::Error;
#[cfg(feature = "ssr")]
use tracing::instrument;

/// Nix version as parsed from `nix --version`
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct NixVersion {
    major: u32,
    minor: u32,
    patch: u32,
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum BadNixVersion {
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
    #[error("Parse error: `nix --version` cannot be parsed")]
    Parse(#[from] std::num::ParseIntError),
    #[error("Parse error: `nix --version` cannot be parsed")]
    Command,
}

impl FromStr for NixVersion {
    type Err = BadNixVersion;

    /// Parse the string output of `nix --version` into a [NixVersion]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"nix \(Nix\) (\d+)\.(\d+)\.(\d+)")?;

        let captures = re.captures(s).ok_or(BadNixVersion::Command)?;
        let major = captures[1].parse::<u32>()?;
        let minor = captures[2].parse::<u32>()?;
        let patch = captures[3].parse::<u32>()?;

        Ok(NixVersion {
            major,
            minor,
            patch,
        })
    }
}

/// Get the output of `nix --version`
#[cfg(feature = "ssr")]
#[instrument(name = "version")]
pub async fn run_nix_version() -> Result<NixVersion, ServerFnError> {
    use tokio::process::Command;
    let mut cmd = Command::new("nix");
    cmd.arg("--version");
    let stdout: Vec<u8> = crate::command::run_command(&mut cmd).await?;
    // Utf-8 errors don't matter here because we're just parsing numbers
    let v = NixVersion::from_str(&String::from_utf8_lossy(&stdout))?;
    Ok(v)
}

/// The HTML view for [NixVersion]
impl IntoView for NixVersion {
    fn into_view(self, cx: Scope) -> View {
        view! { cx, <pre>{format!("{}", self)}</pre> }.into_view(cx)
    }
}

/// The String view for [NixVersion]
impl fmt::Display for NixVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(feature = "ssr")]
#[tokio::test]
async fn test_run_nix_version() {
    let nix_version = run_nix_version().await.unwrap();
    println!("Nix version: {}", nix_version);
}

#[cfg(feature = "ssr")]
#[tokio::test]
async fn test_parse_nix_version() {
    assert_eq!(
        NixVersion::from_str("nix (Nix) 2.13.0"),
        Ok(NixVersion {
            major: 2,
            minor: 13,
            patch: 0
        })
    );
    assert_eq!(
        NixVersion::from_str("nix 2.4.0"),
        Err(BadNixVersion::Command)
    );
}
