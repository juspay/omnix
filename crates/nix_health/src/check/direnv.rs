use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use crate::traits::{Check, CheckResult, Checkable};
#[cfg(feature = "ssr")]
use nix_rs::{env, info};

/// Check if direnv is in use
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Direnv {
    enable: bool,
}

impl Default for Direnv {
    fn default() -> Self {
        Self { enable: true }
    }
}

#[cfg(feature = "ssr")]
impl Checkable for Direnv {
    fn check(&self, _nix_info: &info::NixInfo, nix_env: &env::NixEnv) -> Option<Check> {
        if !self.enable {
            return None;
        }
        // This check is only relevant if the flake is local
        let local_path = nix_env
            .current_flake
            .as_ref()
            .and_then(|url| url.as_local_path())?;
        let suggestion = format!("Install direnv <https://zero-to-flakes.com/direnv/#setup> and run `direnv allow` under `{}`", local_path.display());
        let check = Check {
            title: "Direnv activated".to_string(),
            // TODO: Show direnv path
            info: format!("Local flake: {:?}", local_path),
            result: match direnv_active(local_path) {
                Ok(true) => CheckResult::Green,
                Ok(false) => CheckResult::Red {
                    msg: "direnv is not active".to_string(),
                    suggestion,
                },
                Err(e) => CheckResult::Red {
                    msg: format!("Unable to check direnv status: {}", e),
                    suggestion,
                },
            },
        };
        Some(check)
    }
}

/// Check if direnv was already activated in [project_dir]
#[cfg(feature = "ssr")]
pub fn direnv_active(project_dir: &std::path::Path) -> anyhow::Result<bool> {
    let cmd = "direnv status | grep 'Found RC allowed true'";
    // TODO: Don't use `sh`
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .current_dir(project_dir)
        .output()?;
    Ok(output.status.success())
}
