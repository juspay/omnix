//! Rust module for `nix --version`
use regex::Regex;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::{fmt, str::FromStr, sync::LazyLock};
use thiserror::Error;
use tokio::sync::OnceCell;

use tracing::instrument;

use crate::command::{NixCmd, NixCmdError};

/// Simple version triple (major.minor.patch)
#[derive(Clone, Copy, PartialOrd, PartialEq, Eq, Ord, Debug)]
pub struct VersionSpec {
    /// Major version
    pub major: u32,
    /// Minor version
    pub minor: u32,
    /// Patch version
    pub patch: u32,
}

/// Nix version as parsed from `nix --version`, capturing both version and installation type
#[derive(Clone, Copy, Debug, SerializeDisplay, DeserializeFromStr)]
pub enum NixVersion {
    /// Official Nix installation: "nix (Nix) 2.28.4" or "2.28.4"
    Official(VersionSpec),
    /// Determinate Systems Nix: "nix (Determinate Nix 3.8.5) 2.30.2"
    DeterminateSystems {
        /// The Determinate Systems version (e.g., 3.8.5)
        det_sys_version: VersionSpec,
        /// The underlying Nix version (e.g., 2.30.2)
        nix_version: VersionSpec,
    },
}

impl NixVersion {
    /// Get the effective Nix version (the actual Nix version being used)
    pub fn nix_version(&self) -> VersionSpec {
        match self {
            NixVersion::Official(version) => *version,
            NixVersion::DeterminateSystems { nix_version, .. } => *nix_version,
        }
    }

    /// Check if this is a Determinate Systems installation
    pub fn is_determinate_systems(&self) -> bool {
        matches!(self, NixVersion::DeterminateSystems { .. })
    }

    /// Get the installation type
    pub fn installation_type(&self) -> NixInstallationType {
        match self {
            NixVersion::Official(_) => NixInstallationType::Official,
            NixVersion::DeterminateSystems { .. } => NixInstallationType::DeterminateSystems,
        }
    }
}

impl PartialEq for NixVersion {
    fn eq(&self, other: &Self) -> bool {
        self.nix_version() == other.nix_version()
    }
}

impl Eq for NixVersion {}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for NixVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NixVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.nix_version().cmp(&other.nix_version())
    }
}

/// Type of Nix installation (derived from NixVersion)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NixInstallationType {
    /// Official Nix installation
    Official,
    /// Determinate Systems Nix
    DeterminateSystems,
}

impl std::fmt::Display for NixInstallationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NixInstallationType::Official => write!(f, "official"),
            NixInstallationType::DeterminateSystems => write!(f, "determinate-systems"),
        }
    }
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

impl VersionSpec {
    /// Parse a version string like "2.28.4" into a VersionSpec
    fn parse_version_string(s: &str) -> Result<Self, BadNixVersion> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(BadNixVersion::Command);
        }

        let major = parts[0].parse::<u32>()?;
        let minor = parts[1].parse::<u32>()?;
        let patch = parts[2].parse::<u32>()?;

        Ok(VersionSpec {
            major,
            minor,
            patch,
        })
    }
}

impl std::fmt::Display for VersionSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

static DET_SYS_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^nix \(Determinate Nix ([\d.]+)\) ([\d.]+)$").unwrap());
static OFFICIAL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?:nix \(Nix\) )?([\d.]+)$").unwrap());

impl FromStr for NixVersion {
    type Err = BadNixVersion;

    /// Parse the string output of `nix --version` into a [NixVersion]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Try to match Determinate Systems format: "nix (Determinate Nix 3.8.5) 2.30.2"
        if let Some(captures) = DET_SYS_REGEX.captures(s) {
            return Ok(NixVersion::DeterminateSystems {
                det_sys_version: VersionSpec::parse_version_string(&captures[1])?,
                nix_version: VersionSpec::parse_version_string(&captures[2])?,
            });
        }

        // Try to match official format: "nix (Nix) 2.28.4" or plain format: "2.28.4"
        if let Some(captures) = OFFICIAL_REGEX.captures(s) {
            return Ok(NixVersion::Official(VersionSpec::parse_version_string(
                &captures[1],
            )?));
        }

        Err(BadNixVersion::Command)
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
        match self {
            NixVersion::Official(version) => write!(f, "{}", version),
            NixVersion::DeterminateSystems { nix_version, .. } => write!(f, "{}", nix_version),
        }
    }
}

#[tokio::test]
async fn test_run_nix_version() {
    let nix_version = NixVersion::from_nix(&NixCmd::default()).await.unwrap();
    println!("Nix version: {}", nix_version);
}

#[tokio::test]
async fn test_parse_nix_version() {
    // Test official Nix format
    assert_eq!(
        NixVersion::from_str("nix (Nix) 2.13.0"),
        Ok(NixVersion::Official(VersionSpec {
            major: 2,
            minor: 13,
            patch: 0
        }))
    );

    // Test simple version format (treated as official)
    assert_eq!(
        NixVersion::from_str("2.13.0"),
        Ok(NixVersion::Official(VersionSpec {
            major: 2,
            minor: 13,
            patch: 0
        }))
    );

    // Test Determinate Systems format
    assert_eq!(
        NixVersion::from_str("nix (Determinate Nix 3.6.6) 2.29.0"),
        Ok(NixVersion::DeterminateSystems {
            det_sys_version: VersionSpec {
                major: 3,
                minor: 6,
                patch: 6
            },
            nix_version: VersionSpec {
                major: 2,
                minor: 29,
                patch: 0
            }
        })
    );

    // Test installation type detection
    let official = NixVersion::from_str("nix (Nix) 2.28.4").unwrap();
    assert_eq!(official.installation_type(), NixInstallationType::Official);
    assert!(!official.is_determinate_systems());

    let det_sys = NixVersion::from_str("nix (Determinate Nix 3.8.5) 2.30.2").unwrap();
    assert_eq!(
        det_sys.installation_type(),
        NixInstallationType::DeterminateSystems
    );
    assert!(det_sys.is_determinate_systems());

    // Test version comparison between Official and DeterminateSystems
    let official_v2_30 = NixVersion::from_str("2.30.0").unwrap();
    let det_sys_v2_30 = NixVersion::from_str("nix (Determinate Nix 3.8.5) 2.30.0").unwrap();
    let official_v2_28 = NixVersion::from_str("2.28.0").unwrap();

    // Same underlying Nix version should be equal
    assert_eq!(official_v2_30, det_sys_v2_30);

    // Both should be greater than older version
    assert!(official_v2_30 > official_v2_28);
    assert!(det_sys_v2_30 > official_v2_28);
}
