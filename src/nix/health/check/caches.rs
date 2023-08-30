use leptos::*;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::nix::{
    config::ConfigVal,
    health::{
        report::{Report, WithDetails},
        traits::Check,
    },
    info,
    system,
};

/// Check that [crate::nix::config::NixConfig::substituters] is set to a good value.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Caches(ConfigVal<Vec<Url>>);

impl Check for Caches {
    fn check(info: &info::NixInfo, _: &system::SysInfo) -> Self {
        Caches(info.nix_config.substituters.clone())
    }
    fn name(&self) -> &'static str {
        "Nix Caches in use"
    }
    fn report(&self) -> Report<WithDetails> {
        let val = &self.0.value;
        if val.contains(&Url::parse("https://cache.nixos.org").unwrap()) {
            // TODO: Hardcoding this to test failed reports
            if val.contains(&Url::parse("https://nammayatri.cachix.org").unwrap()) {
                Report::Green
            } else {
                Report::Red(WithDetails {
                    msg: "You are missing the nammayatri cache".into(),
                    suggestion: "Run 'nix run nixpkgs#cachix use nammayatri".into(),
                })
            }
        } else {
            Report::Red(WithDetails {
                msg: "You are missing the official cache".into(),
                suggestion: "Try looking in /etc/nix/nix.conf".into(),
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
