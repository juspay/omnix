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
pub struct Caches(ConfigVal<Vec<String>>);

impl Check for Caches {
    fn check(info: &info::NixInfo) -> Self {
        Caches(info.nix_config.substituters.clone())
    }
    fn name(&self) -> &'static str {
        "Nix Caches in use"
    }
    fn report(&self) -> Report<WithDetails> {
        let val = &self.0.value;
        if val.contains(&"https://cache.nixos.org/".to_string()) {
            // TODO: Hardcoding this to test failed reports
            if val.contains(&"https://nammayatri.cachix.org/".to_string()) {
                Report::Green
            } else {
                Report::Red(WithDetails {
                    msg: "You are missing the nammayatri cache",
                    suggestion: "Run 'nix run nixpkgs#cachix use nammayatri",
                })
            }
        } else {
            Report::Red(WithDetails {
                msg: "You are missing the official cache",
                suggestion: "Try looking in /etc/nix/nix.conf",
            })
        }
    }
}

impl IntoView for Caches {
    fn into_view(self, cx: Scope) -> View {
        view! { cx, <div>"The following caches are in use:" {self.0.into_view(cx)}</div> }
            .into_view(cx)
    }
}
