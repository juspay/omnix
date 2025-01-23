//! Rust module for `nix --version`
use regex::Regex;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::{cmp::Ordering, fmt, str::FromStr};
use thiserror::Error;
use tokio::sync::OnceCell;

use tracing::instrument;

use crate::command::{NixCmd, NixCmdError};

/// Nix version as parsed from `nix --version`
#[derive(Clone, Copy, Debug, SerializeDisplay, DeserializeFromStr)]
pub struct NixVersion {
    /// Major version
    pub major: u32,
    /// Minor version
    pub minor: Option<u32>,
    /// Patch version
    pub patch: Option<u32>,
}

/// Error type for parsing `nix --version`
#[derive(Error, Debug, Clone, PartialEq)]
pub enum BadNixVersion {
    /// Regex error
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    /// Parse error
    #[error("Parse error (regex): `nix --version` cannot be parsed")]
    Parse(#[from] std::num::ParseIntError),

    /// Command error
    #[error("Parse error (int): `nix --version` cannot be parsed")]
    Command,
}

impl FromStr for NixVersion {
    type Err = BadNixVersion;

    /// Parse the string output of `nix --version` into a [NixVersion]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // NOTE: The parser is lenient in allowing pure nix version (produced
        // by [Display] instance), so as to work with serde_with instances.
        let re = Regex::new(r"(?:nix \(Nix\) )?(\d+)(?:\.(\d+))?(?:\.(\d+))?")?;

        let captures = re.captures(s).ok_or(BadNixVersion::Command)?;
        let major = captures[1].parse::<u32>()?;
        let minor = captures
            .get(2)
            .map(|m| m.as_str().parse::<u32>())
            .transpose()?;
        let patch = captures
            .get(3)
            .map(|m| m.as_str().parse::<u32>())
            .transpose()?;

        Ok(NixVersion {
            major,
            minor,
            patch,
        })
    }
}

static NIX_VERSION: OnceCell<Result<NixVersion, NixCmdError>> = OnceCell::const_new();

impl NixVersion {
    /// Get the once version of `NixVersion`.
    #[instrument(name = "show-config(once)")]
    pub async fn get() -> &'static Result<NixVersion, NixCmdError> {
        NIX_VERSION
            .get_or_init(|| async {
                let cmd = NixCmd::default();
                let nix_ver = NixVersion::from_nix(&cmd).await?;
                Ok(nix_ver)
            })
            .await
    }
    /// Get the output of `nix --version`
    #[instrument(name = "version")]
    pub async fn from_nix(cmd: &NixCmd) -> Result<NixVersion, super::command::NixCmdError> {
        let v = cmd.run_with_args_expecting_fromstr(&["--version"]).await?;
        Ok(v)
    }

    fn normalized_minor(&self) -> u32 {
        self.minor.unwrap_or(0)
    }

    fn normalized_patch(&self) -> u32 {
        self.patch.unwrap_or(0)
    }
}

impl PartialEq for NixVersion {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major
            && self.normalized_minor() == other.normalized_minor()
            && self.normalized_patch() == other.normalized_patch()
    }
}

impl Eq for NixVersion {}

impl PartialOrd for NixVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NixVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.normalized_minor().cmp(&other.normalized_minor()) {
                Ordering::Equal => self.normalized_patch().cmp(&other.normalized_patch()),
                ord => ord,
            },
            ord => ord,
        }
    }
}

/// The String view for [NixVersion]
impl fmt::Display for NixVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}",
            self.major,
            self.normalized_minor(),
            self.normalized_patch()
        )
    }
}

#[tokio::test]
async fn test_run_nix_version() {
    let nix_version = NixVersion::from_nix(&NixCmd::default()).await.unwrap();
    println!("Nix version: {}", nix_version);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parse() {
        assert_eq!(
            NixVersion::from_str("nix (Nix) 2.13.0").unwrap(),
            NixVersion {
                major: 2,
                minor: Some(13),
                patch: Some(0)
            }
        );
    }

    #[test]
    fn test_version_equality() {
        assert_eq!(
            NixVersion::from_str("2").unwrap(),
            NixVersion::from_str("2.0").unwrap()
        );
        assert_eq!(
            NixVersion::from_str("2.1").unwrap(),
            NixVersion::from_str("2.1.0").unwrap()
        );
    }

    #[test]
    fn test_version_ordering() {
        let v1 = NixVersion::from_str("2").unwrap();
        let v2 = NixVersion::from_str("2.1").unwrap();
        let v3 = NixVersion::from_str("2.1.1").unwrap();

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1 < v3);
    }
}
