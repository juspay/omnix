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

/// Check that [crate::nix::config::NixConfig::experimental_features] is set to a good value.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlakeEnabled(ConfigVal<Vec<String>>);

impl Check for FlakeEnabled {
    fn check(info: &info::NixInfo) -> Self {
        FlakeEnabled(info.nix_config.experimental_features.clone())
    }
    fn name(&self) -> &'static str {
        "Flakes Enabled"
    }
    fn report(&self) -> Report<WithDetails> {
        let val = &self.0.value;
        if val.contains(&"flakes".to_string()) && val.contains(&"nix-command".to_string()) {
            Report::Green
        } else {
            Report::Red(WithDetails {
                msg: "Nix flakes are not enabled",
                suggestion: "See https://nixos.wiki/wiki/Flakes#Enable_flakes",
            })
        }
    }
}

impl IntoView for FlakeEnabled {
    fn into_view(self, cx: Scope) -> View {
        view! { cx, <span>"experimental-features: " {self.0}</span> }.into_view(cx)
    }
}
