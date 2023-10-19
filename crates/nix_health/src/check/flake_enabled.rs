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
        _: Option<nix_rs::flake::url::FlakeUrl>,
    ) -> Vec<Check> {
        let val = &nix_info.nix_config.experimental_features.value;
        let check = Check {
            title: "Flakes Enabled".to_string(),
            info: format!("experimental-features = {}", val.join(" ")),
            result: if val.contains(&"flakes".to_string())
                && val.contains(&"nix-command".to_string())
            {
                CheckResult::Green
            } else {
                CheckResult::Red {
                    msg: "Nix flakes are not enabled".into(),
                    suggestion: "See https://nixos.wiki/wiki/Flakes#Enable_flakes".into(),
                }
            },
            required: true,
        };
        vec![check]
    }
}
