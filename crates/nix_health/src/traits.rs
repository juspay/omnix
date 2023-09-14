use nix_rs::{env, info};
use serde::{Deserialize, Serialize};

/// Types that can do specific "health check" for Nix
pub trait Checkable {
    /// Run and create the health check
    fn check(&self, nix_info: &info::NixInfo, nix_env: &env::NixEnv) -> Option<Check>;
}

/// A health check
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Check {
    /// A user-facing title of this check
    ///
    /// This value is expected to be unique across all checks.
    pub title: String,

    /// The user-facing information used to conduct this check
    /// TODO: Use Markdown
    pub info: String,

    /// The result of running this check
    pub result: CheckResult,
}

/// The result of a health [Check]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum CheckResult {
    /// The check passed
    Green,

    /// The check failed
    Red {
        /// TODO: Use markdown
        msg: String,
        /// TODO: Use markdown
        suggestion: String,
    },
}

impl CheckResult {
    /// When the check is green (ie., healthy)
    pub fn green(&self) -> bool {
        matches!(self, Self::Green)
    }
}
