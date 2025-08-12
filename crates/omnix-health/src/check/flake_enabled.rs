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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::CheckResult;
    use nix_rs::{
        config::{ConfigVal, NixConfig, TrustedUserValue},
        flake::system::{Arch, System},
        info::NixInfo,
        version::NixVersion,
    };
    use url::Url;

    fn mock_config_val<T: Clone>(value: T) -> ConfigVal<T> {
        ConfigVal {
            value: value.clone(),
            default_value: value,
            description: "Mock config value".to_string(),
        }
    }

    fn mock_nix_config_without_flakes() -> NixConfig {
        NixConfig {
            cores: mock_config_val(4),
            experimental_features: mock_config_val(vec!["auto-allocate-uids".to_string()]),
            extra_platforms: mock_config_val(vec![]),
            flake_registry: mock_config_val("https://flake-registry.example.com".to_string()),
            max_jobs: mock_config_val(8),
            substituters: mock_config_val(vec![Url::parse("https://cache.nixos.org/").unwrap()]),
            system: mock_config_val(System::Linux(Arch::X86_64)),
            trusted_users: mock_config_val(vec![TrustedUserValue::User("root".to_string())]),
        }
    }

    fn mock_nix_config_with_flakes() -> NixConfig {
        NixConfig {
            cores: mock_config_val(4),
            experimental_features: mock_config_val(vec![
                "flakes".to_string(),
                "nix-command".to_string(),
            ]),
            extra_platforms: mock_config_val(vec![]),
            flake_registry: mock_config_val("https://flake-registry.example.com".to_string()),
            max_jobs: mock_config_val(8),
            substituters: mock_config_val(vec![Url::parse("https://cache.nixos.org/").unwrap()]),
            system: mock_config_val(System::Linux(Arch::X86_64)),
            trusted_users: mock_config_val(vec![TrustedUserValue::User("root".to_string())]),
        }
    }

    async fn mock_nix_info(is_detsys: bool, has_flakes_config: bool) -> NixInfo {
        let version = NixVersion {
            major: 2,
            minor: 29,
            patch: 0,
            is_detsys,
        };
        let config = if has_flakes_config {
            mock_nix_config_with_flakes()
        } else {
            mock_nix_config_without_flakes()
        };
        NixInfo::new(version, config).await.unwrap()
    }

    #[tokio::test]
    async fn test_flakes_enabled_with_detsys() {
        let checker = FlakeEnabled::default();
        let nix_info = mock_nix_info(true, false).await;
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
        let nix_info = mock_nix_info(false, true).await;
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
        let nix_info = mock_nix_info(false, false).await;
        let results = checker.check(&nix_info, None);

        assert_eq!(results.len(), 1);
        let (_, check) = &results[0];
        assert_eq!(check.title, "Flakes Enabled");
        assert_eq!(check.info, "experimental-features = auto-allocate-uids");
        assert!(matches!(check.result, CheckResult::Red { .. }));
    }
}
