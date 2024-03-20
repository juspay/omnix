use nix_rs::{flake::url::FlakeUrl, info};
use semver::VersionReq;
use serde::{Deserialize, Serialize};

use crate::traits::{Check, CheckResult, Checkable};

/// Check if direnv is installed
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default, rename_all = "kebab-case")]
pub struct Direnv {
    pub(crate) enable: bool,
    /// Whether to produce [Check::required] checks
    pub(crate) required: bool,
}

impl Default for Direnv {
    fn default() -> Self {
        Self {
            enable: true,
            required: false,
        }
    }
}

impl Checkable for Direnv {
    // TODO: This code flow is confusing; refactor for legibility.
    fn check(&self, _nix_info: &info::NixInfo, flake_url: Option<FlakeUrl>) -> Vec<Check> {
        let mut checks = vec![];
        if !self.enable {
            return checks;
        }

        let direnv_install_result = direnv_crate::DirenvInstall::detect();

        let direnv_install_check = install_check(&direnv_install_result, self.required);
        let direnv_installed = direnv_install_check.result.green();
        checks.push(direnv_install_check);

        if !direnv_installed {
            return checks;
        }

        // FIXME: Avoid unwrap, by refactoring code flow.
        let direnv_install = direnv_install_result.as_ref().unwrap();

        let direnv_version = version_check(direnv_install);
        let direnv_version_green = direnv_version.result.green();
        checks.push(direnv_version);

        // If direnv is installed, check for version and then allowed_check
        if direnv_version_green {
            // This check is currently only relevant if the flake is local and an `.envrc` exists.
            match flake_url.as_ref().and_then(|url| url.as_local_path()) {
                None => {}
                Some(local_path) => {
                    if local_path.join(".envrc").exists() {
                        checks.push(allowed_check(direnv_install, local_path, self.required));
                    }
                }
            }
        }

        checks
    }
}

/// [Check] that direnv was installed.
fn install_check(
    direnv_install: &anyhow::Result<direnv_crate::DirenvInstall>,
    required: bool,
) -> Check {
    Check {
        title: "Direnv installation".to_string(),
        info: format!(
            "direnv location = {:?}",
            direnv_install.as_ref().ok().map(|s| &s.bin_path)
        ),
        result: match direnv_install {
            Ok(_direnv_status) => CheckResult::Green,
            Err(e) => CheckResult::Red {
                msg: format!("Unable to locate direnv ({})", e),
                suggestion: "Install direnv <https://nixos.asia/en/direnv#setup>".to_string(),
            },
        },
        required,
    }
}

/// [Check] that direnv version >= 2.33.0 for `direnv status --json` support
fn version_check(direnv_install: &direnv_crate::DirenvInstall) -> Check {
    let req = VersionReq::parse(">=2.33.0").unwrap();
    let suggestion = format!("Upgrade direnv to {}", req);
    let direnv_version = &direnv_install.version;
    Check {
        title: "Direnv version".to_string(),
        info: format!("direnv version = {:?}", direnv_version),
        // Use semver to compare versions
        result: if req.matches(direnv_version) {
            CheckResult::Green
        } else {
            CheckResult::Red {
                msg: format!("direnv version {} is not supported", direnv_version),
                suggestion,
            }
        },
        required: false,
    }
}

/// [Check] that direnv was allowed on the local flake
fn allowed_check(
    direnv_install: &direnv_crate::DirenvInstall,
    local_flake: &std::path::Path,
    required: bool,
) -> Check {
    let suggestion = format!("Run `direnv allow` under `{}`", local_flake.display());
    let direnv_allowed = direnv_install
        .status(local_flake)
        .map(|status| status.state.is_allowed());
    Check {
        title: "Direnv allowed".to_string(),
        info: format!("Local flake: {:?} (has .envrc and is allowed)", local_flake),
        result: match direnv_allowed {
            Ok(true) => CheckResult::Green,
            Ok(false) => CheckResult::Red {
                msg: "direnv was not allowed on this project".to_string(),
                suggestion,
            },
            Err(e) => CheckResult::Red {
                msg: format!("Unable to check direnv status: {}", e),
                suggestion,
            },
        },
        required,
    }
}

/// TODO: Move this to a separate crate, called `direnv`
/// TODO: Don't use anyhow::Result in library crates
mod direnv_crate {
    use semver::Version;
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

    impl DirenvInstall {
        /// Detect user's direnv installation
        pub fn detect() -> anyhow::Result<Self> {
            let bin_path = which::which("direnv")?;
            let version = get_direnv_version(&bin_path)?;
            Ok(DirenvInstall { bin_path, version })
        }

        /// Return the `direnv status` on the given project directory
        pub fn status(&self, project_dir: &std::path::Path) -> anyhow::Result<DirenvStatus> {
            DirenvStatus::new(&self.bin_path, project_dir)
        }
    }

    /// Get the version of direnv
    fn get_direnv_version(direnv_bin: &PathBuf) -> anyhow::Result<Version> {
        let output = std::process::Command::new(direnv_bin)
            .args(["--version"])
            .output()?;
        let out = String::from_utf8_lossy(&output.stdout);
        Ok(Version::parse(out.trim())?)
    }

    /// Information about the direnv status
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct DirenvStatus {
        pub config: DirenvConfig,
        pub state: DirenvState,
    }

    impl DirenvStatus {
        /// Run `direnv status` and parse the output, for the given project directory.
        fn new(direnv_bin: &PathBuf, dir: &std::path::Path) -> anyhow::Result<Self> {
            let output = std::process::Command::new(direnv_bin)
                .args(["status", "--json"])
                .current_dir(dir)
                .output()?;
            let out = String::from_utf8_lossy(&output.stdout);
            let status = DirenvStatus::from_json(&out)?;
            Ok(status)
        }

        /// Parse the output of `direnv status --json`
        fn from_json(json: &str) -> anyhow::Result<Self> {
            let status: DirenvStatus = serde_json::from_str(json)?;
            Ok(status)
        }
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
    /// TODO: Represent 0/1/2 values in a 3-value enum
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
}
