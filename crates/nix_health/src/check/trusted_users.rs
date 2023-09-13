use nix_rs::{config::ConfigVal, info, system};
use os_info;
use serde::{Deserialize, Serialize};

use crate::{
    report::{Report, WithDetails},
    traits::Check,
};

/// Check that [crate::nix::config::NixConfig::trusted_users] is set to a good value.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrustedUsers {
    pub trusted_users: ConfigVal<Vec<String>>,
    current_user: String,
    os: os_info::Type,
}

impl Check for TrustedUsers {
    fn check(nix_info: &info::NixInfo, sys_info: &system::SysInfo) -> Self {
        TrustedUsers {
            trusted_users: nix_info.nix_config.trusted_users.clone(),
            current_user: sys_info.current_user.clone(),
            os: sys_info.os,
        }
    }
    fn name(&self) -> &'static str {
        "Trusted users"
    }
    fn report(&self) -> Report<WithDetails> {
        let trusted_users = &self.trusted_users.value;
        let current_user = &self.current_user;
        let os = self.os;
        if trusted_users.contains(current_user) {
            Report::Green
        } else if os == os_info::Type::NixOS {
            Report::Red(WithDetails {
                msg: "$USER not present in trusted_users".into(),
                suggestion:
                    "Add `nix.trustedUsers = [ \"root\" \"<$USER>\" ];` to your `configuration.nix`"
                        .into(),
            })
        } else {
            Report::Red(WithDetails {
                msg: "$USER not present in trusted_users".into(),
                suggestion: "Run 'echo \"trusted-users = root $USER\" | sudo tee -a /etc/nix/nix.conf && sudo pkill nix-daemon'".into(),
            })
        }
    }
}
