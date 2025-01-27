use std::str::FromStr;

use nix_rs::version_spec::NixVersionReq;

use nix_rs::info;
use serde::{Deserialize, Serialize};

use crate::traits::*;

/// Check that [nix_rs::version::NixVersion] is set to a good value.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(default, rename_all = "kebab-case")]
pub struct NixVersionCheck {
    pub supported: NixVersionReq,
}

impl Default for NixVersionCheck {
    fn default() -> Self {
        NixVersionCheck {
            supported: NixVersionReq::from_str(">=2.16.0").unwrap(),
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

        let is_supported = self.supported.specs.iter().all(|spec| spec.matches(val));

        let supported_version_check = Check {
            title: "Nix Version is supported".to_string(),
            info: format!("nix version = {}", val),
            result: if is_supported {
                CheckResult::Green
            } else {
                CheckResult::Red {
                    msg: format!(
                        "Your Nix version ({}) doesn't satisfy the supported bounds: {}",
                        val, self.supported
                    ),
                    suggestion: "To use a specific version of Nix, see <https://nixos.asia/en/howto/nix-package>"
                        .into(),
                }
            },
            required: true,
        };

        vec![("supported-nix-versions", supported_version_check)]
    }
}
