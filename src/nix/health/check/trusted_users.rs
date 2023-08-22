use std::env;

use leptos::*;
use serde::{Deserialize, Serialize};

use crate::nix::{
    config::ConfigVal,
    health::{
        report::{Report, WithDetails},
        traits::Check,
    },
    info,
};

/// Check that [crate::nix::config::NixConfig::substituters] is set to a good value.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrustedUsers(ConfigVal<Vec<String>>);

impl Check for TrustedUsers {
    fn check(info: &info::NixInfo) -> Self {
        TrustedUsers(info.nix_config.trusted_users.clone())
    }
    fn name(&self) -> &'static str {
        "Trusted users"
    }
    fn report(&self) -> Report<WithDetails> {
        let val = &self.0.value;
        let current_user = env::var("USER");
        if let Ok(user) = current_user {
            if val.contains(&user) {
                Report::Green
            } else {
                Report::Red(WithDetails {
                    msg: "$USER not present in trusted_users".into(),
                    suggestion: "Run 'echo \"trusted-users = root $USER\" | sudo tee -a /etc/nix/nix.conf && sudo pkill nix-daemon'".into(),
                })
            }
        } else {
            Report::Red(WithDetails {
                msg: "$USER environment variable not set".into(),
                suggestion: "Run 'export USER=$(whoami)'".into(),
            })
        }
    }
}

impl IntoView for TrustedUsers {
    fn into_view(self, cx: Scope) -> View {
        view! { cx, <div>"The following trusted_users are present:" {self.0.into_view(cx)}</div> }
            .into_view(cx)
    }
}
