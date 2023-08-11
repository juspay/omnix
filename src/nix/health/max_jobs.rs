use serde::{Deserialize, Serialize};

use crate::nix::info;

use super::{Check, Report};

// [NixConfig::max_job]]
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Clone)]
pub struct MaxJobs(i32);

impl Check for MaxJobs {
    fn check(info: info::NixInfo) -> Self {
        MaxJobs(info.nix_config.max_jobs.value)
    }
    fn name(&self) -> &'static str {
        "Max Jobs"
    }
    fn report(&self) -> Report {
        if self > &MaxJobs(12) {
            Report::Green
        } else {
            Report::Red {
                msg: "You are using only 1 core for nix builds",
                suggestion: "Try editing /etc/nix/nix.conf",
            }
        }
    }
}
