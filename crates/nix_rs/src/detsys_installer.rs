use serde::{Deserialize, Serialize};

use std::{fmt::Display, io::ErrorKind, path::Path};

use regex::Regex;
use thiserror::Error;

// The version of Detsys/nix-installer
#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Clone)]
pub struct DetSysNixInstallerVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
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

impl DetSysNixInstallerVersion {
    pub fn get_version(executable_path: &Path) -> Result<Self, BadInstallerVersion> {
        let output = std::process::Command::new(executable_path)
            .arg("--version")
            .output()
            .map_err(BadInstallerVersion::Command)?;
        let output = String::from_utf8(output.stdout)?;
        let re = Regex::new(r"(?:nix-installer )?(\d+)\.(\d+)\.(\d+)")?;

        let captures = re.captures(&output).ok_or(BadInstallerVersion::Command(std::io::Error::new(ErrorKind::InvalidData, "Failed to capture regex")))?;
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
