use nix_rs::{config::ConfigVal, info, system};
use serde::{Deserialize, Serialize};

use crate::{
    report::{Report, WithDetails},
    traits::Check,
};

/// Check that [crate::config::NixConfig::max_jobs] is set to a good value.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MaxJobs(pub ConfigVal<i32>);

impl Check for MaxJobs {
    fn check(nix_info: &info::NixInfo, _sys_info: &system::SysInfo) -> Self {
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
