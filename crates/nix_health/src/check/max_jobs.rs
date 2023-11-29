use nix_rs::info;
use serde::{Deserialize, Serialize};

use crate::traits::*;

/// Check that [nix_rs::config::NixConfig::max_jobs] is set to a good value.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct MaxJobs {}

impl Checkable for MaxJobs {
    fn check(
        &self,
        nix_info: &info::NixInfo,
        _: Option<nix_rs::flake::url::FlakeUrl>,
    ) -> Vec<Check> {
        let max_jobs = nix_info.nix_config.max_jobs.value;
        let check = Check {
            title: "Max Jobs".to_string(),
            info: format!("max-jobs = {}", max_jobs),
            result: if max_jobs > 1 {
                CheckResult::Green
            } else {
                CheckResult::Red {
                    msg: "You are using only 1 CPU core for nix builds".into(),
                    suggestion: format!(
                        "Set `max-jobs = auto` in {}",
                        nix_info.nix_env.os.nix_config_label()
                    ),
                }
            },
            required: true,
        };
        vec![check]
    }
}
