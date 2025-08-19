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

        let (_flakes_enabled, info_msg, result) = match nix_info.nix_version.installation_type {
            NixInstallationType::DeterminateSystems => (
                true, // Determinate Systems Nix has flakes enabled by default
                "Flakes enabled via Determinate Systems Nix".to_string(),
                CheckResult::Green,
            ),
            NixInstallationType::Official => {
                let flakes_enabled =
                    val.contains(&"flakes".to_string()) && val.contains(&"nix-command".to_string());
                let info_msg = format!("experimental-features = {}", val.join(" "));
                let result = match flakes_enabled {
                    true => CheckResult::Green,
                    false => CheckResult::Red {
                        msg: "Nix flakes are not enabled".into(),
                        suggestion: "See https://nixos.wiki/wiki/Flakes#Enable_flakes".into(),
                    },
                };
                (flakes_enabled, info_msg, result)
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::CheckResult;
    use nix_rs::{config::NixConfig, info::NixInfo, version::NixVersion};

    fn mock_nix_config(experimental_features: &str) -> NixConfig {
        // Mock minimal nix.conf: experimental-features = flakes nix-command
        let features_array = experimental_features
            .split_whitespace()
            .map(|s| format!("\"{}\"", s))
            .collect::<Vec<_>>()
            .join(",");
        let config_json = format!(
            r#"{{
            "experimental-features": {{ "value": [{}], "defaultValue": [], "description": "" }},
            "cores": {{ "value": 1, "defaultValue": 1, "description": "" }},
            "extra-platforms": {{ "value": [], "defaultValue": [], "description": "" }},
            "flake-registry": {{ "value": "", "defaultValue": "", "description": "" }},
            "max-jobs": {{ "value": 1, "defaultValue": 1, "description": "" }},
            "substituters": {{ "value": [], "defaultValue": [], "description": "" }},
            "system": {{ "value": "x86_64-linux", "defaultValue": "x86_64-linux", "description": "" }},
            "trusted-users": {{ "value": [], "defaultValue": [], "description": "" }}
        }}"#,
            features_array
        );
        serde_json::from_str(&config_json).unwrap()
    }

    async fn mock_nix_info(
        installation_type: NixInstallationType,
        has_flakes_config: bool,
    ) -> NixInfo {
        let version = NixVersion {
            major: 2,
            minor: 29,
            patch: 0,
            installation_type,
        };
        let experimental_features = if has_flakes_config {
            "flakes nix-command"
        } else {
            "auto-allocate-uids"
        };
        let config = mock_nix_config(experimental_features);
        NixInfo::new(version, config).await.unwrap()
    }

    #[tokio::test]
    async fn test_flakes_enabled_with_detsys() {
        let checker = FlakeEnabled::default();
        let nix_info = mock_nix_info(NixInstallationType::DeterminateSystems, false).await;
        let results = checker.check(&nix_info, None);

        let (_, check) = &results[0];
        assert!(matches!(check.result, CheckResult::Green));
    }

    #[tokio::test]
    async fn test_flakes_enabled_with_regular_nix_and_config() {
        let checker = FlakeEnabled::default();
        let nix_info = mock_nix_info(NixInstallationType::Official, true).await;
        let results = checker.check(&nix_info, None);

        let (_, check) = &results[0];
        assert!(matches!(check.result, CheckResult::Green));
    }

    #[tokio::test]
    async fn test_flakes_disabled_with_regular_nix() {
        let checker = FlakeEnabled::default();
        let nix_info = mock_nix_info(NixInstallationType::Official, false).await;
        let results = checker.check(&nix_info, None);

        let (_, check) = &results[0];
        assert!(matches!(check.result, CheckResult::Red { .. }));
    }
}
