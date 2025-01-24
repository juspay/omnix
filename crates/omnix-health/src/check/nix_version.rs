use nix_rs::version_spec::NixVersionReq;

use nix_rs::info;
use serde::{Deserialize, Serialize};

use crate::traits::*;

/// Check that [nix_rs::version::NixVersion] is set to a good value.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(default, rename_all = "kebab-case")]
pub struct NixVersionCheck {
    pub supported: String,
}

impl Default for NixVersionCheck {
    fn default() -> Self {
        NixVersionCheck {
            supported: ">=2.16.0".to_string(),
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

        let supported_req = NixVersionReq::parse(&self.supported).unwrap();

        let supported_version_check = Check {
            title: "Supported Nix Versions".to_string(),
            info: format!("nix version = {}", val),
            result: if supported_req.specs.iter().all(|spec| spec.matches(val)) {
                CheckResult::Green
            } else {
                CheckResult::Red {
                    msg: format!(
                        "Your Nix version ({}) doesn't satisfy the supported bounds: {}",
                        val, self.supported
                    ),
                    // TODO: Link to a blog post here that lists various ways to use a specific version of Nix
                    suggestion: "Set `nix.package` in home-manager to the desired Nix version"
                        .into(),
                }
            },
            required: true,
        };

        vec![("supported-nix-versions", supported_version_check)]
    }
}
