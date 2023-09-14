use nix_rs::{env, info};
use serde::{Deserialize, Serialize};

/// Types that implement health check with reports
pub trait Checkable {
    /// Run and create the health check
    fn check(&self, nix_info: &info::NixInfo, nix_env: &env::NixEnv) -> Option<Check>;
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Check {
    /// A user-facing title of this check
    pub title: &'static str,

    /// The information used to conduct this check
    /// TODO: Should be Markdown
    pub info: String,

    /// The result of running this check
    pub result: CheckResult,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum CheckResult {
    Green,
    /// TODO: Use markdown
    Red {
        msg: String,
        suggestion: String,
    },
}

