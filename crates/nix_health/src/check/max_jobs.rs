use std::fmt::Display;

use nix_rs::{config::ConfigVal, env, info};
use serde::{Deserialize, Serialize};

use crate::{
    report::{Report, WithDetails},
    traits::Check,
};

/// Check that [nix_rs::config::NixConfig::max_jobs] is set to a good value.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MaxJobs(pub ConfigVal<i32>);

impl Check for MaxJobs {
    fn check(nix_info: &info::NixInfo, _nix_env: &env::NixEnv) -> Self {
        MaxJobs(nix_info.nix_config.max_jobs.clone())
    }
    fn name(&self) -> &'static str {
        "Max Jobs"
    }
    fn report(&self) -> Report<WithDetails> {
        if self.0.value > 1 {
            Report::Green
        } else {
            Report::Red(WithDetails {
                msg: "You are using only 1 core for nix builds".into(),
                suggestion: "Try editing /etc/nix/nix.conf".into(),
            })
        }
    }
}

impl Display for MaxJobs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "max-jobs = {}", self.0.value)
    }
}
