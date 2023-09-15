use nix_rs::{env, info};
use serde::{Deserialize, Serialize};

use crate::traits::*;

/// Check that [crate::nix::config::NixConfig::trusted_users] is set to a good value.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct TrustedUsers {}

impl Checkable for TrustedUsers {
    fn check(&self, nix_info: &info::NixInfo, nix_env: &env::NixEnv) -> Option<Check> {
        let val = &nix_info.nix_config.trusted_users.value;
        let current_user = &nix_env.current_user;
        let result = if val.contains(current_user) {
            CheckResult::Green
        } else {
            let msg = format!("User '{}' not present in trusted_users", current_user);
            let suggestion = if nix_env.os.has_configuration_nix() {
                format!(
                    r#"Add `nix.trustedUsers = [ "root" "{}" ];` to your {} `configuration.nix`"#,
                    current_user, nix_env.os,
                )
            } else {
                format!(
                    r#"Run 'echo "trusted-users = root {}" | sudo tee -a /etc/nix/nix.conf && sudo pkill nix-daemon'"#,
                    current_user
                )
            };
            CheckResult::Red { msg, suggestion }
        };
        let check = Check {
            title: "Trusted Users".to_string(),
            info: format!("trusted-users = {}", val.join(" ")),
            result,
        };
        Some(check)
    }
}
