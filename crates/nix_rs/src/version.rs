//! Rust module for `nix --version`
use regex::Regex;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::{fmt, str::FromStr};
use thiserror::Error;
#[cfg(feature = "ssr")]
use tracing::instrument;

/// Nix version as parsed from `nix --version`
#[derive(Clone, PartialOrd, PartialEq, Eq, Debug, SerializeDisplay, DeserializeFromStr)]
pub struct NixVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum BadNixVersion {
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
    #[error("Parse error (regex): `nix --version` cannot be parsed")]
    Parse(#[from] std::num::ParseIntError),
    #[error("Parse error (int): `nix --version` cannot be parsed")]
    Command,
}

impl FromStr for NixVersion {
    type Err = BadNixVersion;

    /// Parse the string output of `nix --version` into a [NixVersion]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // NOTE: The parser is lenient in allowing pure nix version (produced
        // by [Display] instance), so as to work with serde_with instances.
        let re = Regex::new(r"(?:nix \(Nix\) )?(\d+)\.(\d+)\.(\d+)")?;

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

impl NixVersion {
    /// Get the output of `nix --version`
    #[cfg(feature = "ssr")]
    #[instrument(name = "version")]
    pub async fn from_nix(
        nix_cmd: &super::command::NixCmd,
    ) -> Result<NixVersion, super::command::NixCmdError> {
        let v = nix_cmd
            .run_with_args_expecting_fromstr(&["--version"])
            .await?;
        Ok(v)
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
    let nix_version = NixVersion::from_nix(&crate::command::NixCmd::default())
        .await
        .unwrap();
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

    // Parse simple nix version
    assert_eq!(
        NixVersion::from_str("2.13.0"),
        Ok(NixVersion {
            major: 2,
            minor: 13,
            patch: 0
        })
    );
}
