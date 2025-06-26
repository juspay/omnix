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
        if !self.enable {
            return checks;
        }

        // Only check on macOS by default
        if !matches!(nix_info.nix_env.os, nix_rs::env::OS::MacOS { .. }) {
            return checks;
        }

        let homebrew_install_result = detect_homebrew();
        checks.push((
            "homebrew-check",
            installation_check(&homebrew_install_result, self.required),
        ));

        checks
    }
}

/// Result of Homebrew detection
#[derive(Debug)]
pub struct HomebrewInstall {
    pub bin_path: std::path::PathBuf,
}

/// Error types for Homebrew detection
#[derive(Debug, thiserror::Error)]
pub enum HomebrewError {
    #[error("Homebrew not found in PATH")]
    NotFound,
}

/// Detect if Homebrew is installed
fn detect_homebrew() -> Result<HomebrewInstall, HomebrewError> {
    which_strict("brew")
        .map(|bin_path| HomebrewInstall { bin_path })
        .ok_or(HomebrewError::NotFound)
}

/// A string containing step-by-step removal commands and migration advice.
fn homebrew_removal_instructions() -> String {
    r#"To completely remove Homebrew from your system:

- **Uninstall Homebrew and all packages:**
   /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/uninstall.sh)"

For a safer migration, consider using 'brew list' to inventory your packages before removal, then install equivalents via Nix."#.to_string()
}

/// Create a [Check] for Homebrew installation
fn installation_check(
    homebrew_result: &Result<HomebrewInstall, HomebrewError>,
    required: bool,
) -> Check {
    let nix_setup_url = "https://github.com/juspay/nixos-unified-template";

    Check {
        title: "Homebrew installation".to_string(),
        info: format!(
            "Homebrew detection result: {}",
            homebrew_result
                .as_ref()
                .map(|h| format!("Found at {:?}", h.bin_path))
                .unwrap_or_else(|e| format!("{}", e))
        ),
        result: match homebrew_result {
            Ok(homebrew) => CheckResult::Red {
                msg: format!(
                    "Homebrew is installed at {:?}. Consider using Nix for better reproducibility",
                    homebrew.bin_path
                ),
                suggestion: format!(
                    "While Homebrew works fine, managing packages with Nix provides better reproducibility and integration. See <{}>\n\n{}",
                    nix_setup_url,
                    homebrew_removal_instructions()
                ),
            },
            Err(HomebrewError::NotFound) => CheckResult::Green,
        },
        required,
    }
}
