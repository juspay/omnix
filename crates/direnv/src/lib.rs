use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::path::PathBuf;

/// Information about a local direnv installation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DirenvInstall {
    /// Path to the direnv binary
    pub bin_path: PathBuf,
    /// Version of the installed direnv
    pub version: Version,
}

#[derive(thiserror::Error, Debug)]
pub enum DirenvInstallError {
    #[error("Cannot find direnv binary")]
    DirenvWhichError(#[from] which::Error),

    #[error("Direnv command error: {0}")]
    DirenvCmdError(#[from] std::io::Error),

    #[error("Cannot parse direnv version: {0}")]
    DirenvVersionError(#[from] semver::Error),
}

impl DirenvInstall {
    /// Detect user's direnv installation
    pub fn detect() -> Result<Self, DirenvInstallError> {
        let bin_path = which::which("direnv")?;
        let output = std::process::Command::new(&bin_path)
            .args(["--version"])
            .output()?;
        let out = String::from_utf8_lossy(&output.stdout);
        let version = Version::parse(out.trim())?;
        Ok(DirenvInstall { bin_path, version })
    }

    /// Return the `direnv status` on the given project directory
    pub fn status(&self, project_dir: &std::path::Path) -> Result<DirenvStatus, DirenvStatusError> {
        let min_version = VersionReq::parse(">=2.33.0").unwrap(); // --json was added in latter versions
        if min_version.matches(&self.version) {
            DirenvStatus::new(&self.bin_path, project_dir)
        } else {
            Err(DirenvStatusError::DirenvVersionError(min_version))
        }
    }
}

/// Information about the direnv status
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DirenvStatus {
    pub config: DirenvConfig,
    pub state: DirenvState,
}

impl DirenvStatus {
    /// Run `direnv status` and parse the output, for the given project directory.
    fn new(direnv_bin: &PathBuf, dir: &std::path::Path) -> Result<Self, DirenvStatusError> {
        let output = std::process::Command::new(direnv_bin)
            .args(["status", "--json"])
            .current_dir(dir)
            .output()?;
        let out = String::from_utf8_lossy(&output.stdout);
        let status: DirenvStatus = serde_json::from_str(&out)?;
        Ok(status)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DirenvStatusError {
    #[error("Direnv command error: {0}")]
    DirenvCmdError(#[from] std::io::Error),

    #[error("Cannot parse direnv status JSON: {0}")]
    DirenvStatusError(#[from] serde_json::Error),

    #[error("Direnv version is too old; you need direnv {0}")]
    DirenvVersionError(semver::VersionReq),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DirenvConfig {
    /// Path to the config folder of direnv
    #[serde(rename = "ConfigDir")]
    pub config_dir: PathBuf,
    /// Path to the direnv binary
    #[serde(rename = "SelfPath")]
    pub self_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DirenvState {
    /// Information about the .envrc found in the current directory
    #[serde(rename = "foundRC")]
    pub found_rc: Option<DirenvRC>,
    /// Information about the .envrc that is currently allowed using `direnv allow`
    #[serde(rename = "loadedRC")]
    pub loaded_rc: Option<DirenvRC>,
}

impl DirenvState {
    /// Check if the .envrc file is allowed
    pub fn is_allowed(&self) -> bool {
        self.found_rc
            .as_ref()
            .map_or(false, |rc| rc.allowed == AllowedStatus::Allowed)
    }
}

/// Information about the .envrc file
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DirenvRC {
    pub allowed: AllowedStatus,
    /// Path to the .envrc file
    pub path: PathBuf,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone)]
#[repr(u8)]
/// Can be 0, 1 or 2
/// 0: Allowed
/// 1: NotAllowed
/// 2: Denied
pub enum AllowedStatus {
    Allowed = 0,
    NotAllowed = 1,
    Denied = 2,
}
