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
                if flakes_enabled {
                    (
                        flakes_enabled,
                        format!("experimental-features = {}", val.join(" ")),
                        CheckResult::Green,
                    )
                } else {
                    (
                        flakes_enabled,
                        format!("experimental-features = {}", val.join(" ")),
                        CheckResult::Red {
                            msg: "Nix flakes are not enabled".into(),
                            suggestion: "See https://nixos.wiki/wiki/Flakes#Enable_flakes".into(),
                        },
                    )
                }
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
    use nix_rs::{
        config::{ConfigVal, NixConfig},
        flake::system::{Arch, System},
        info::NixInfo,
        version::NixVersion,
    };

    fn mock_nix_config(experimental_features: Vec<String>) -> NixConfig {
        NixConfig {
            experimental_features: ConfigVal {
                value: experimental_features.clone(),
                default_value: experimental_features,
                description: String::new(),
            },
            // Required fields - Rust doesn't allow partial struct construction
            cores: ConfigVal {
                value: 1,
                default_value: 1,
                description: String::new(),
            },
            extra_platforms: ConfigVal {
                value: vec![],
                default_value: vec![],
                description: String::new(),
            },
            flake_registry: ConfigVal {
                value: String::new(),
                default_value: String::new(),
                description: String::new(),
            },
            max_jobs: ConfigVal {
                value: 1,
                default_value: 1,
                description: String::new(),
            },
            substituters: ConfigVal {
                value: vec![],
                default_value: vec![],
                description: String::new(),
            },
            system: ConfigVal {
                value: System::Linux(Arch::X86_64),
                default_value: System::Linux(Arch::X86_64),
                description: String::new(),
            },
            trusted_users: ConfigVal {
                value: vec![],
                default_value: vec![],
                description: String::new(),
            },
        }
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
            vec!["flakes".to_string(), "nix-command".to_string()]
        } else {
            vec!["auto-allocate-uids".to_string()]
        };
        let config = mock_nix_config(experimental_features);
        NixInfo::new(version, config).await.unwrap()
    }

    #[tokio::test]
    async fn test_flakes_enabled_with_detsys() {
        let checker = FlakeEnabled::default();
        let nix_info = mock_nix_info(NixInstallationType::DeterminateSystems, false).await;
        let results = checker.check(&nix_info, None);

        assert_eq!(results.len(), 1);
        let (_, check) = &results[0];
        assert_eq!(check.title, "Flakes Enabled");
        assert_eq!(check.info, "Flakes enabled via Determinate Systems Nix");
        assert!(matches!(check.result, CheckResult::Green));
    }

    #[tokio::test]
    async fn test_flakes_enabled_with_regular_nix_and_config() {
        let checker = FlakeEnabled::default();
        let nix_info = mock_nix_info(NixInstallationType::Official, true).await;
        let results = checker.check(&nix_info, None);

        assert_eq!(results.len(), 1);
        let (_, check) = &results[0];
        assert_eq!(check.title, "Flakes Enabled");
        assert_eq!(check.info, "experimental-features = flakes nix-command");
        assert!(matches!(check.result, CheckResult::Green));
    }

    #[tokio::test]
    async fn test_flakes_disabled_with_regular_nix() {
        let checker = FlakeEnabled::default();
        let nix_info = mock_nix_info(NixInstallationType::Official, false).await;
        let results = checker.check(&nix_info, None);

        assert_eq!(results.len(), 1);
        let (_, check) = &results[0];
        assert_eq!(check.title, "Flakes Enabled");
        assert_eq!(check.info, "experimental-features = auto-allocate-uids");
        assert!(matches!(check.result, CheckResult::Red { .. }));
    }
}
