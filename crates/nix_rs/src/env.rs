//! Information about the environment in which Nix will run
use std::{fmt::Display, path::Path};

use crate::detsys_installer::{BadInstallerVersion, DetSysNixInstallerVersion};
use bytesize::ByteSize;
use os_info;
use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::instrument;

/// The environment in which Nix operates
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NixEnv {
    /// Current user ($USER)
    pub current_user: String,
    /// Current user groups
    pub current_user_groups: Vec<String>,
    /// Underlying OS in which Nix runs
    pub os: OS,
    /// Total disk space of the volume where /nix exists.
    ///
    /// This is either root volume or the dedicated /nix volume.
    pub total_disk_space: ByteSize,
    /// Total memory
    pub total_memory: ByteSize,
    /// The installer used to install Nix
    pub installer: NixInstaller,
}

impl NixEnv {
    /// Determine [NixEnv] on the user's system

    #[instrument]
    pub async fn detect() -> Result<NixEnv, NixEnvError> {
        use sysinfo::{DiskExt, SystemExt};
        tracing::debug!("Detecting Nix environment");
        let os = OS::detect().await;
        tokio::task::spawn_blocking(|| {
            let current_user = std::env::var("USER")?;
            let sys = sysinfo::System::new_with_specifics(
                sysinfo::RefreshKind::new().with_disks_list().with_memory(),
            );
            let total_disk_space = to_bytesize(get_nix_disk(&sys)?.total_space());
            let total_memory = to_bytesize(sys.total_memory());
            let current_user_groups = get_current_user_groups()?;
            let installer = NixInstaller::detect()?;
            Ok(NixEnv {
                current_user,
                current_user_groups,
                os,
                total_disk_space,
                total_memory,
                installer,
            })
        })
        .await
        .unwrap()
    }
}

/// Get the current user's groups
fn get_current_user_groups() -> Result<Vec<String>, NixEnvError> {
    let output = Command::new("groups")
        .output()
        .map_err(NixEnvError::GroupsError)?;
    let group_info = &String::from_utf8_lossy(&output.stdout);
    Ok(group_info
        .as_ref()
        .split_whitespace()
        .map(|v| v.to_string())
        .collect())
}

/// Get the disk where /nix exists
fn get_nix_disk(sys: &sysinfo::System) -> Result<&sysinfo::Disk, NixEnvError> {
    use sysinfo::{DiskExt, SystemExt};
    let by_mount_point: std::collections::HashMap<&Path, &sysinfo::Disk> = sys
        .disks()
        .iter()
        .map(|disk| (disk.mount_point(), disk))
        .collect();
    // Lookup /nix first, then /.
    by_mount_point
        .get(Path::new("/nix"))
        .copied()
        .or_else(|| by_mount_point.get(Path::new("/")).copied())
        .ok_or(NixEnvError::NoDisk)
}

/// The system under which Nix is installed and operates
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OS {
    /// On macOS
    MacOS {
        /// Using nix-darwin
        nix_darwin: bool,
        arch: MacOSArch,
    },
    /// On NixOS
    NixOS,
    /// Nix is individually installed on Linux or macOS
    Other(os_info::Type),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MacOSArch {
    Arm64(AppleEmulation),
    Other(Option<String>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppleEmulation {
    None,
    Rosetta,
}

impl AppleEmulation {
    pub fn new() -> Self {
        use is_proc_translated::is_proc_translated;
        if is_proc_translated() {
            AppleEmulation::Rosetta
        } else {
            AppleEmulation::None
        }
    }
}

impl Default for AppleEmulation {
    fn default() -> Self {
        Self::new()
    }
}

impl MacOSArch {
    pub fn from(os_arch: Option<&str>) -> MacOSArch {
        match os_arch {
            Some("arm64") => MacOSArch::Arm64(AppleEmulation::new()),
            other => MacOSArch::Other(other.map(|s| s.to_string())),
        }
    }
}

// The [Display] instance affects how [OS] is displayed to the app user
impl Display for OS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OS::MacOS {
                nix_darwin,
                arch: _,
            } => {
                if *nix_darwin {
                    write!(f, "nix-darwin")
                } else {
                    write!(f, "macOS")
                }
            }
            OS::NixOS => write!(f, "NixOS"),
            OS::Other(os_type) => write!(f, "{}", os_type),
        }
    }
}

impl OS {
    pub async fn detect() -> Self {
        let os_info = tokio::task::spawn_blocking(os_info::get).await.unwrap();
        let os_type = os_info.os_type();
        let arch = MacOSArch::from(os_info.architecture());
        async fn is_symlink(file_path: &str) -> std::io::Result<bool> {
            let metadata = tokio::fs::symlink_metadata(file_path).await?;
            Ok(metadata.file_type().is_symlink())
        }
        match os_type {
            os_info::Type::Macos => {
                // To detect that we are on NixDarwin, we check if /etc/nix/nix.conf
                // is a symlink (which nix-darwin manages like NixOS does)
                let nix_darwin = is_symlink("/etc/nix/nix.conf").await.unwrap_or(false);
                OS::MacOS { nix_darwin, arch }
            }
            os_info::Type::NixOS => OS::NixOS,
            _ => OS::Other(os_type),
        }
    }

    /// Return the label for nix-darwin or NixOS system
    pub fn nix_system_config_label(&self) -> Option<String> {
        // TODO: This should return Markdown
        match self {
            OS::MacOS {
                nix_darwin,
                arch: _,
            } if *nix_darwin => Some("nix-darwin configuration".to_string()),
            OS::NixOS => Some("nixos configuration".to_string()),
            _ => None,
        }
    }

    /// Return the label for where Nix is configured
    pub fn nix_config_label(&self) -> String {
        self.nix_system_config_label()
            .unwrap_or("/etc/nix/nix.conf".to_string())
    }
}

/// The installer used to install Nix (applicable only for non-NixOS systems)
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum NixInstaller {
    /// The installer from <https://github.com/DeterminateSystems/nix-installer>
    DetSys { version: DetSysNixInstallerVersion },
    /// Either offical installer or from a different package manager
    Other,
}

impl Display for NixInstaller {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NixInstaller::DetSys { version } => {
                write!(f, "DetSys nix-installer ({})", version)
            }
            NixInstaller::Other => write!(f, "Unknown installer"),
        }
    }
}

impl NixInstaller {
    pub fn detect() -> Result<Self, NixEnvError> {
        let nix_installer_path = Path::new("/nix/nix-installer");
        if nix_installer_path.exists() {
            Ok(NixInstaller::DetSys {
                version: DetSysNixInstallerVersion::get_version(nix_installer_path)?,
            })
        } else {
            Ok(NixInstaller::Other)
        }
    }
}

/// Errors while trying to fetch [NixEnv]

#[derive(thiserror::Error, Debug)]
pub enum NixEnvError {
    #[error("Cannot find $USER: {0}")]
    UserError(#[from] std::env::VarError),

    #[error("Failed to fetch groups: {0}")]
    GroupsError(std::io::Error),

    #[error("Unable to find root disk or /nix volume")]
    NoDisk,

    #[error("Failed to detect Nix installer: {0}")]
    InstallerError(#[from] BadInstallerVersion),
}

/// Convert bytes to a closest [ByteSize]
///
/// Useful for displaying disk space and memory which are typically in GBs / TBs

fn to_bytesize(bytes: u64) -> ByteSize {
    let kb = bytes / 1024;
    let mb = kb / 1024;
    let gb = mb / 1024;
    if gb > 0 {
        ByteSize::gib(gb)
    } else if mb > 0 {
        ByteSize::mib(mb)
    } else if kb > 0 {
        ByteSize::kib(kb)
    } else {
        ByteSize::b(bytes)
    }
}

/// Test for [to_bytesize]

#[test]
fn test_to_bytesize() {
    assert_eq!(to_bytesize(0), ByteSize::b(0));
    assert_eq!(to_bytesize(1), ByteSize::b(1));
    assert_eq!(to_bytesize(1023), ByteSize::b(1023));
    assert_eq!(to_bytesize(1024), ByteSize::kib(1));
    assert_eq!(to_bytesize(1024 * 1024), ByteSize::mib(1));
    assert_eq!(to_bytesize(1024 * 1024 * 1024), ByteSize::gib(1));
}
