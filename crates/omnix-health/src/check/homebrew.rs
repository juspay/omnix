use nix_rs::{flake::url::FlakeUrl, info::NixInfo};
use omnix_common::check::which_strict;
use serde::{Deserialize, Serialize};

use crate::traits::{Check, CheckResult, Checkable};

/// Check if Homebrew is installed
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(default, rename_all = "kebab-case")]
pub struct Homebrew {
    pub(crate) enable: bool,
    pub(crate) required: bool,
}

impl Checkable for Homebrew {
    fn check(
        &self,
        nix_info: &NixInfo,
        _flake_url: Option<&FlakeUrl>,
    ) -> Vec<(&'static str, Check)> {
        let mut checks = vec![];

        if self.enable && matches!(nix_info.nix_env.os, nix_rs::env::OS::MacOS { .. }) {
            checks.push((
                "homebrew-check",
                installation_check(&HomebrewInstall::detect(), self.required),
            ));
        }

        checks
    }
}

/// Information about user's homebrew installation.
#[derive(Debug)]
pub struct HomebrewInstall {
    /// The path to the Homebrew binary.
    pub bin_path: std::path::PathBuf,
}

impl HomebrewInstall {
    /// Detect homebrew installation.
    pub fn detect() -> Option<Self> {
        which_strict("brew").map(|bin_path| HomebrewInstall { bin_path })
    }
}

/// Create a [Check] for Homebrew installation
fn installation_check(homebrew_result: &Option<HomebrewInstall>, required: bool) -> Check {
    let nix_setup_url = "https://github.com/juspay/nixos-unified-template";

    Check {
        title: "Homebrew installation".to_string(),
        info: format!(
            "Homebrew binary: {}",
            homebrew_result
                .as_ref()
                .map(|h| format!("Found at {:?}", h.bin_path))
                .unwrap_or_else(|| "Not found".to_string())
        ),
        result: match homebrew_result {
            Some(homebrew) => CheckResult::Red {
                msg: format!(
                    "Homebrew is installed at {:?}. Consider using Nix for better reproducibility",
                    homebrew.bin_path
                ),
                suggestion: format!(
                    "Managing packages with Nix, rather than Homebrew, provides better reproducibility and integration. See <{}>\n\n{}",
                    nix_setup_url,
                    HOMEBREW_REMOVAL_INSTRUCTIONS
                ),
            },
            None => CheckResult::Green,
        },
        required,
    }
}

/// A string containing step-by-step removal commands and migration advice.
const HOMEBREW_REMOVAL_INSTRUCTIONS: &str = r#"To completely remove Homebrew from your system:

- **Uninstall Homebrew and all packages:**
   /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/uninstall.sh)"

For a safer migration, consider using 'brew list' to inventory your packages before removal, then install equivalents via Nix."#;
