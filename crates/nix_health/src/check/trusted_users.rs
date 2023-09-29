use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use crate::traits::*;

/// Check that [crate::nix::config::NixConfig::trusted_users] is set to a good value.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct TrustedUsers {}

#[cfg(feature = "ssr")]
impl Checkable for TrustedUsers {
    fn check(&self, nix_info: &nix_rs::info::NixInfo, nix_env: &nix_rs::env::NixEnv) -> Vec<Check> {
        let val = &nix_info.nix_config.trusted_users.value;
        let current_user = &nix_env.current_user;
        let result = if val.contains(current_user) {
            CheckResult::Green
        } else {
            let msg = format!("User '{}' not present in trusted_users", current_user);
            let suggestion = match nix_env.os.nix_system_config_label() {
                Some(conf_label) => format!(
                    r#"Add `nix.trustedUsers = [ "root" "{}" ];` to your {}"#,
                    current_user, conf_label,
                ),
                None => format!(
                    r#"Run 'echo "trusted-users = root {}" | sudo tee -a /etc/nix/nix.conf && sudo pkill nix-daemon'"#,
                    current_user
                ),
            };
            CheckResult::Red { msg, suggestion }
        };
        let check = Check {
            title: "Trusted Users".to_string(),
            info: format!("trusted-users = {}", val.join(" ")),
            result,
            required: true,
        };
        vec![check]
    }
}
