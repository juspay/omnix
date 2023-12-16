use std::collections::HashSet;

use nix_rs::config::TrustedUserValue;
use serde::{Deserialize, Serialize};

use crate::traits::*;

/// Check that [crate::nix::config::NixConfig::trusted_users] is set to a good value.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct TrustedUsers {}

impl Checkable for TrustedUsers {
    fn check(
        &self,
        nix_info: &nix_rs::info::NixInfo,
        _: Option<nix_rs::flake::url::FlakeUrl>,
    ) -> Vec<Check> {
        let result = if is_current_user_trusted(nix_info) {
            CheckResult::Green
        } else {
            let current_user = &nix_info.nix_env.current_user;
            let msg = format!("User '{}' not present in trusted_users", current_user);
            let suggestion = match nix_info.nix_env.os.nix_system_config_label() {
                Some(conf_label) => format!(
                    r#"Add `nix.trustedUsers = [ "root" "{}" ];` to your {}"#,
                    current_user, conf_label,
                ),
                None => format!(
                    r#"Set `trusted-users = root {}` in /etc/nix/nix.conf and then restart the Nix daemon using `sudo pkill nix-daemon`"#,
                    current_user
                ),
            };
            CheckResult::Red { msg, suggestion }
        };
        let check = Check {
            title: "Trusted Users".to_string(),
            info: format!(
                "trusted-users = {}",
                TrustedUserValue::display_original(&nix_info.nix_config.trusted_users.value)
            ),
            result,
            required: true,
        };
        vec![check]
    }
}

/// Check that [crate::nix::config::NixConfig::trusted_users] is set to a good
/// value such that the current user is trusted.
fn is_current_user_trusted(nix_info: &nix_rs::info::NixInfo) -> bool {
    let current_user = &nix_info.nix_env.current_user;
    let current_user_groups: HashSet<&String> =
        nix_info.nix_env.current_user_groups.iter().collect();
    nix_info
        .nix_config
        .trusted_users
        .value
        .iter()
        .any(|x| match x {
            TrustedUserValue::Group(x) => current_user_groups.contains(&x),
            TrustedUserValue::User(x) => x == current_user,
            TrustedUserValue::All => true,
        })
}
