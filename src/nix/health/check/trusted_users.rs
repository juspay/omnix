use leptos::*;
use serde::{Deserialize, Serialize};

use crate::nix::{
    config::ConfigVal,
    health::{
        report::{Report, WithDetails},
        traits::Check,
    },
    info, system,
};

/// Check that [crate::nix::config::NixConfig::substituters] is set to a good value.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrustedUsers {
    trusted_users: ConfigVal<Vec<String>>,
    current_user: String,
}

impl Check for TrustedUsers {
    fn check(nix_info: &info::NixInfo, sys_info: &system::SysInfo) -> Self {
        TrustedUsers {
            trusted_users: nix_info.nix_config.trusted_users.clone(),
            current_user: sys_info.current_user.clone(),
        }
    }
    fn name(&self) -> &'static str {
        "Trusted users"
    }
    fn report(&self) -> Report<WithDetails> {
        let trusted_users = &self.trusted_users.value;
        let current_user = &self.current_user;
        // tracing::info!("{:?} {:?}", val, current_user);
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

impl IntoView for TrustedUsers {
    fn into_view(self, cx: Scope) -> View {
        view! { cx,
            <div>"The following trusted_users are present:" {self.trusted_users.into_view(cx)}</div>
        }
            .into_view(cx)
    }
}
