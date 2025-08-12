//! Rust module for `nix --version`
use regex::Regex;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::{fmt, str::FromStr};
use thiserror::Error;
use tokio::sync::OnceCell;

use tracing::instrument;

use crate::command::{NixCmd, NixCmdError};

/// Nix installation type
#[derive(Clone, Copy, PartialOrd, PartialEq, Eq, Debug)]
pub enum NixInstallationType {
    /// Official Nix installation
    Official,
    /// Determinate Systems Nix installation
    DeterminateSystems,
}

/// Nix version as parsed from `nix --version`
#[derive(Clone, Copy, PartialOrd, PartialEq, Eq, Debug, SerializeDisplay, DeserializeFromStr)]
pub struct NixVersion {
    /// Major version
    pub major: u32,
    /// Minor version
    pub minor: u32,
    /// Patch version
    pub patch: u32,
    /// Nix installation type
    pub installation_type: NixInstallationType,
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
        let re = Regex::new(r"(?:nix \(.*?\) )?(\d+)\.(\d+)\.(\d+)$")?;

        let captures = re.captures(s).ok_or(BadNixVersion::Command)?;
        let major = captures[1].parse::<u32>()?;
        let minor = captures[2].parse::<u32>()?;
        let patch = captures[3].parse::<u32>()?;

        // Determine the installation type based on the version string
        let installation_type = if s.contains("Determinate") {
            NixInstallationType::DeterminateSystems
        } else {
            NixInstallationType::Official
        };

        Ok(NixVersion {
            major,
            minor,
            patch,
            installation_type,
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
        let v = cmd
            .run_with_args_expecting_fromstr(&[], &["--version"])
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

#[tokio::test]
async fn test_run_nix_version() {
    let nix_version = NixVersion::from_nix(&NixCmd::default()).await.unwrap();
    println!("Nix version: {}", nix_version);
}

#[tokio::test]
async fn test_parse_nix_version() {
    assert_eq!(
        NixVersion::from_str("nix (Nix) 2.13.0"),
        Ok(NixVersion {
            major: 2,
            minor: 13,
            patch: 0,
            installation_type: NixInstallationType::Official
        })
    );

    // Parse simple nix version
    assert_eq!(
        NixVersion::from_str("2.13.0"),
        Ok(NixVersion {
            major: 2,
            minor: 13,
            patch: 0,
            installation_type: NixInstallationType::Official
        })
    );

    // Parse Determinate Nix Version
    assert_eq!(
        NixVersion::from_str("nix (Determinate Nix 3.6.6) 2.29.0"),
        Ok(NixVersion {
            major: 2,
            minor: 29,
            patch: 0,
            installation_type: NixInstallationType::DeterminateSystems
        })
    );
}
