use nix_rs::{info, version::NixInstallationType};
use serde::{Deserialize, Serialize};

use crate::traits::*;

/// Check that [nix_rs::config::NixConfig::experimental_features] is set to a good value.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct FlakeEnabled {}

impl Checkable for FlakeEnabled {
    fn check(
        &self,
        nix_info: &info::NixInfo,
        _: Option<&nix_rs::flake::url::FlakeUrl>,
    ) -> Vec<(&'static str, Check)> {
        let val = &nix_info.nix_config.experimental_features.value;

        // Check if flakes are enabled either through configuration or installation type
        let flakes_enabled = match nix_info.installation_type() {
            NixInstallationType::DeterminateSystems => {
                // Determinate Systems Nix has flakes enabled by default
                true
            }
            NixInstallationType::Official => {
                // Official Nix requires explicit configuration
                val.contains(&"flakes".to_string()) && val.contains(&"nix-command".to_string())
            }
        };

        let info_msg = match nix_info.installation_type() {
            NixInstallationType::DeterminateSystems => {
                "Determinate Systems Nix (flakes enabled by default)".to_string()
            }
            NixInstallationType::Official => {
                format!("experimental-features = {}", val.join(" "))
            }
        };

        let check = Check {
            title: "Flakes Enabled".to_string(),
            info: info_msg,
            result: if flakes_enabled {
                CheckResult::Green
            } else {
                CheckResult::Red {
                    msg: "Nix flakes are not enabled".into(),
                    suggestion: "See https://nixos.wiki/wiki/Flakes#Enable_flakes".into(),
                }
            },
            required: true,
        };

        vec![("flake-enabled", check)]
    }
}
