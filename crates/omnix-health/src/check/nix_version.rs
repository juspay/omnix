use nix_rs::version::NixVersion;

use nix_rs::info;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

use crate::traits::*;

/// Check that [nix_rs::version::NixVersion] is set to a good value.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(default, rename_all = "kebab-case")]
pub struct NixVersionCheck {
    pub min_required: NixVersion,
    pub supported: Vec<VersionReq>,
}

impl Default for NixVersionCheck {
    fn default() -> Self {
        NixVersionCheck {
            min_required: NixVersion {
                major: 2,
                minor: 16,
                patch: 0,
            },
            supported: vec![VersionReq::parse(">=2.16.0").unwrap()],
        }
    }
}

impl Checkable for NixVersionCheck {
    fn check(
        &self,
        nix_info: &info::NixInfo,
        _: Option<&nix_rs::flake::url::FlakeUrl>,
    ) -> Vec<(&'static str, Check)> {
        let val = &nix_info.nix_version;
        let min_version_check = Check {
            title: "Minimum Nix Version".to_string(),
            info: format!("nix version = {}", val),
            result: if val >= &self.min_required {
                CheckResult::Green
            } else {
                CheckResult::Red {
                    msg: format!("Your Nix version ({}) is too old; we require at least {}", val, self.min_required),
                    suggestion: "See https://nixos.org/manual/nix/stable/command-ref/new-cli/nix3-upgrade-nix.html".into(),
                }
            },
            required: true,
        };

        let matches_supported = self
            .supported
            .iter()
            .all(|req| Version::parse(&val.to_string()).map_or(false, |v| req.matches(&v)));

        let supported_versions = self
            .supported
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");

        let supported_version_check = Check {
            title: "Supported Nix Versions".to_string(),
            info: format!("nix version = {}", val),
            result: if matches_supported {
                CheckResult::Green
            } else {
                CheckResult::Red {
                    msg: format!(
                        "Your Nix version ({}) doesn't satisfy the supported bounds: {}",
                        val, supported_versions
                    ),
                    // TODO: Link to a blog post here that lists various ways to use a specific version of Nix
                    suggestion: "Set `nix.package` in home-manager to the desired Nix version"
                        .into(),
                }
            },
            required: true,
        };

        vec![
            ("min-nix-version", min_version_check),
            ("supported-nix-versions", supported_version_check),
        ]
    }
}
