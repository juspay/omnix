use leptos::*;
use serde::{Deserialize, Serialize};

use crate::nix::{config::ConfigVal, info};

use super::{Check, Report};

// [NixConfig::max_job]]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MaxJobs(ConfigVal<i32>);

impl Check for MaxJobs {
    fn check(info: &info::NixInfo) -> Self {
        MaxJobs(info.nix_config.max_jobs.clone())
    }
    fn name(&self) -> &'static str {
        "Max Jobs"
    }
    fn report(&self) -> Report {
        // NOTE: Testing Red view, so this is inverted
        if self.0.value < 1 {
            Report::Green
        } else {
            Report::Red {
                msg: "You are using only 1 core for nix builds",
                suggestion: "Try editing /etc/nix/nix.conf",
            }
        }
    }
}

impl IntoView for MaxJobs {
    fn into_view(self, cx: Scope) -> View {
        view! {cx,
            <span>{self.0} " Cores"</span>
        }
        .into_view(cx)
    }
}
