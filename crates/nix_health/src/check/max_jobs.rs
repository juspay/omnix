use nix_rs::{env, info};
use serde::{Deserialize, Serialize};

use crate::traits::{Check, CheckResult, Checkable};

/// Check that [nix_rs::config::NixConfig::max_jobs] is set to a good value.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct MaxJobs {}

impl Checkable for MaxJobs {
    fn check(&self, nix_info: &info::NixInfo, _nix_env: &env::NixEnv) -> Option<Check> {
        let max_jobs = nix_info.nix_config.max_jobs.value;
        let check = Check {
            title: "Max Jobs".to_string(),
            info: format!("max-jobs = {}", max_jobs),
            result: if max_jobs > 1 {
                CheckResult::Green
            } else {
                CheckResult::Red {
                    msg: "You are using only 1 core for nix builds".into(),
                    suggestion: "Try editing /etc/nix/nix.conf".into(),
                }
            },
        };
        Some(check)
    }
}
