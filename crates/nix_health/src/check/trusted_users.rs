use std::fmt::Display;

use nix_rs::{config::ConfigVal, env, info};
use serde::{Deserialize, Serialize};

use crate::{
    report::{Report, WithDetails},
    traits::Check,
};

/// Check that [crate::nix::config::NixConfig::trusted_users] is set to a good value.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrustedUsers {
    pub trusted_users: ConfigVal<Vec<String>>,
    nix_env: env::NixEnv,
}

impl Check for TrustedUsers {
    fn check(nix_info: &info::NixInfo, nix_env: &env::NixEnv) -> Self {
        TrustedUsers {
            trusted_users: nix_info.nix_config.trusted_users.clone(),
            nix_env: nix_env.clone(),
        }
    }
    fn name(&self) -> &'static str {
        "Trusted users"
    }
    fn report(&self) -> Report<WithDetails> {
        let trusted_users = &self.trusted_users.value;
        let current_user = &self.nix_env.current_user;
        if trusted_users.contains(current_user) {
            Report::Green
        } else if self.nix_env.nix_system.has_configuration_nix() {
            Report::Red(WithDetails {
                msg: format!("{} not present in trusted_users", current_user),
                suggestion: format!(
                    r#"Add `nix.trustedUsers = [ "root" "{}" ];` to your `configuration.nix`"#,
                    current_user
                ),
            })
        } else {
            Report::Red(WithDetails {
                msg: format!("{} not present in trusted_users", current_user),
                suggestion: format!(
                    r#"Run 'echo "trusted-users = root {}" | sudo tee -a /etc/nix/nix.conf && sudo pkill nix-daemon'"#,
                    current_user
                ),
            })
        }
    }
}

impl Display for TrustedUsers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "trusted_users = {}", self.trusted_users.value.join(" "))
    }
}
