use leptos::*;
use serde::{Deserialize, Serialize};

use crate::nix::{
    config::ConfigVal,
    health::{
        report::Report,
        traits::{Check, ViewCheck},
    },
    info,
};

// [NixConfig::max_job]]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Caches(ConfigVal<Vec<String>>);

impl Check for Caches {
    fn check(info: &info::NixInfo) -> Self {
        Caches(info.nix_config.substituters.clone())
    }
    fn name(&self) -> &'static str {
        "Nix Caches in use"
    }
    fn report(&self) -> Report {
        // TODO: This should use lenient URL match
        if self
            .0
            .value
            .contains(&"https://cache.nixos.org/".to_string())
        {
            Report::Green
        } else {
            Report::Red {
                msg: "You are missing the official cache",
                suggestion: "Try looking in /etc/nix/nix.conf",
            }
        }
    }
}

impl IntoView for Caches {
    fn into_view(self, cx: Scope) -> View {
        view! { cx,
            <ViewCheck check=self.clone()>
                <div>{self.0.into_view(cx)}</div>
            </ViewCheck>
        }
    }
}
