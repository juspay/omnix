use nix_rs::info;
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

        // Check if flakes are enabled either through config or DetSys installation
        let flakes_enabled = if nix_info.nix_version.is_detsys {
            // Determinate Systems Nix has flakes enabled by default
            true
        } else {
            // Check experimental-features config for regular Nix
            val.contains(&"flakes".to_string()) && val.contains(&"nix-command".to_string())
        };

        let (info_msg, result) = if nix_info.nix_version.is_detsys {
            (
                "Flakes enabled via Determinate Systems Nix".to_string(),
                CheckResult::Green,
            )
        } else if flakes_enabled {
            (
                format!("experimental-features = {}", val.join(" ")),
                CheckResult::Green,
            )
        } else {
            (
                format!("experimental-features = {}", val.join(" ")),
                CheckResult::Red {
                    msg: "Nix flakes are not enabled".into(),
                    suggestion: "See https://nixos.wiki/wiki/Flakes#Enable_flakes".into(),
                },
            )
        };

        let check = Check {
            title: "Flakes Enabled".to_string(),
            info: info_msg,
            result,
            required: true,
        };

        vec![("flake-enabled", check)]
    }
}
