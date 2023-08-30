use leptos::*;
use serde::{Deserialize, Serialize};

use crate::nix::{
    config::ConfigVal,
    health::{
        report::{Report, WithDetails},
        traits::Check,
    },
    info,
    system,
};

/// Check that [crate::nix::config::NixConfig::max_jobs] is set to a good value.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MaxJobs(ConfigVal<i32>);

impl Check for MaxJobs {
    fn check(info: &info::NixInfo, _: &system::SysInfo) -> Self {
        MaxJobs(info.nix_config.max_jobs.clone())
    }
    fn name(&self) -> &'static str {
        "Max Jobs"
    }
    fn report(&self) -> Report<WithDetails> {
        if self.0.value > 1 {
            Report::Green
        } else {
            Report::Red(WithDetails {
                msg: "You are using only 1 core for nix builds".into(),
                suggestion: "Try editing /etc/nix/nix.conf".into(),
            })
        }
    }
}

impl IntoView for MaxJobs {
    fn into_view(self, cx: Scope) -> View {
        view! { cx, <span>"Nix builds are using " {self.0} " cores"</span> }.into_view(cx)
    }
}
