//! Nix info UI

use std::fmt::Display;

use dioxus::prelude::*;
use nix_rs::{config::NixConfig, info::NixInfo, version::NixVersion};

use crate::{
    app::state::AppState,
    widget::{Loader, RefreshButton},
};

/// Nix information
#[component]
pub fn Info(cx: Scope) -> Element {
    let title = "Nix Info";
    let state = AppState::use_state(cx);
    let nix_info = state.nix_info.read();
    let busy = (*nix_info).is_loading_or_refreshing();
    render! {
        h1 { class: "text-5xl font-bold", title }
        RefreshButton {
            busy: busy,
            handler: move |_event| {
                cx.spawn(async move {
                    state.update_nix_info().await;
                });
            }
        }
        div { class: "my-1",
            match (*nix_info).current_value() {
                None => render! { Loader {}},
                Some(v) =>
                    match v {
                        Ok(info) => render! { NixInfoView { info: info.clone() } },
                        Err(err) => render! { "{err}" }
                    }
            }
        }
    }
}

#[component]
fn NixInfoView(cx: Scope, info: NixInfo) -> Element {
    render! {
        div { class: "flex flex-col p-4 space-y-8 bg-white border-2 rounded border-base-400",
            div {
                b { "Nix Version" }
                div { class: "p-1 my-1 rounded bg-primary-50", NixVersionView { version: info.nix_version } }
            }
            div {
                b { "Nix Config" }
                NixConfigView { config: info.nix_config.clone() }
            }
        }
    }
}

#[component]
fn NixVersionView(cx: Scope, version: NixVersion) -> Element {
    render! {a { href: nix_rs::refs::RELEASE_HISTORY, class: "font-mono hover:underline", target: "_blank", "{version}" }}
}

#[component]
fn NixConfigView(cx: Scope, config: NixConfig) -> Element {
    let config_row = |key: &'static str, title: String, children: Element<'a>| {
        render! {
            tr { title: "{title}",
                td { class: "px-4 py-2 font-semibold text-base-700", "{key}" }
                td { class: "px-4 py-2 text-left",
                    code { children }
                }
            }
        }
    };
    render! {
        div { class: "py-1 my-1 rounded bg-primary-50",
            table { class: "text-right",
                tbody {
                    config_row (
                        "Local System",
                        config.system.description.clone(),
                        render! { "{config.system.value}" }
                    ),
                    config_row (
                        "Max Jobs",
                        config.max_jobs.description.clone(),
                        render! {"{config.max_jobs.value}"}
                    ),
                    config_row (
                        "Cores per build",
                        config.cores.description.clone(),
                        render! { "{config.cores.value}" }
                    ),
                    config_row (
                        "Nix Caches",
                        config.substituters.clone().description,
                        render! { ConfigValList { items: &config.substituters.value } }
                    )
                }
            }
        }
    }
}

#[component]
fn ConfigValList<T, 'a>(cx: Scope, items: &'a Vec<T>) -> Element
where
    T: Display,
{
    render! {
        div { class: "flex flex-col space-y-4",
            for item in items {
                li { class: "list-disc", "{item}" }
            }
        }
    }
}
