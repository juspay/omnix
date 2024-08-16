//! DetSys installer detection
// TODO: Move this under 'env' module.
use serde::{Deserialize, Serialize};

use std::{fmt::Display, io::ErrorKind, path::Path, str::FromStr};

use regex::Regex;
use thiserror::Error;

/// The installer from <https://github.com/DeterminateSystems/nix-installer>
#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Clone)]
pub struct DetSysNixInstaller {
    version: DetSysNixInstallerVersion,
}

impl DetSysNixInstaller {
    pub fn detect() -> Result<Option<Self>, BadInstallerVersion> {
        let nix_installer_path = Path::new("/nix/nix-installer");
        if nix_installer_path.exists() {
            Ok(Some(DetSysNixInstaller {
                version: DetSysNixInstallerVersion::get_version(nix_installer_path)?,
            }))
        } else {
            Ok(None)
        }
    }
}

impl Display for DetSysNixInstaller {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DetSys nix-installer ({})", self.version)
    }
}

// The version of Detsys/nix-installer
#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Clone)]
struct DetSysNixInstallerVersion {
    major: u32,
    minor: u32,
    patch: u32,
}

impl Display for DetSysNixInstallerVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Error, Debug)]
pub enum BadInstallerVersion {
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
    #[error("Failed to decode installer output: {0}")]
    Decode(#[from] std::string::FromUtf8Error),
    #[error("Failed to parse installer version: {0}")]
    Parse(#[from] std::num::ParseIntError),
    #[error("Failed to fetch installer version: {0}")]
    Command(std::io::Error),
}

impl FromStr for DetSysNixInstallerVersion {
    type Err = BadInstallerVersion;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"(\d+)\.(\d+)\.(\d+)")?;

        let captures = re
            .captures(s)
            .ok_or(BadInstallerVersion::Command(std::io::Error::new(
                ErrorKind::InvalidData,
                "Failed to capture regex",
            )))?;
        let major = captures[1].parse::<u32>()?;
        let minor = captures[2].parse::<u32>()?;
        let patch = captures[3].parse::<u32>()?;

        Ok(DetSysNixInstallerVersion {
            major,
            minor,
            patch,
        })
    }
}

impl DetSysNixInstallerVersion {
    pub fn get_version(executable_path: &Path) -> Result<Self, BadInstallerVersion> {
        let output = std::process::Command::new(executable_path)
            .arg("--version")
            .output()
            .map_err(BadInstallerVersion::Command)?;
        let version_str = String::from_utf8(output.stdout)?;
        version_str.parse()
    }
}
