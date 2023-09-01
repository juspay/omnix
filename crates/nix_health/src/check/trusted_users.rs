use nix_rs::{config::ConfigVal, info};
use serde::{Deserialize, Serialize};
use std::env;

use crate::{
    report::{Report, WithDetails},
    traits::Check,
};

/// Check that [crate::nix::config::NixConfig::trusted_users] is set to a good value.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrustedUsers {
    trusted_users: ConfigVal<Vec<String>>,
    current_user: String,
}

impl Check for TrustedUsers {
    fn check(nix_info: &info::NixInfo) -> Self {
        TrustedUsers {
            trusted_users: nix_info.nix_config.trusted_users.clone(),
            current_user: env::var("USER").unwrap(),
        }
    }
    fn name(&self) -> &'static str {
        "Trusted users"
    }
    fn report(&self) -> Report<WithDetails> {
        let trusted_users = &self.trusted_users.value;
        let current_user = &self.current_user;
        if trusted_users.contains(current_user) {
            Report::Green
        } else {
            Report::Red(WithDetails {
                msg: "$USER not present in trusted_users".into(),
                suggestion: "Run 'echo \"trusted-users = root $USER\" | sudo tee -a /etc/nix/nix.conf && sudo pkill nix-daemon'".into(),
            })
        }
    }
}
