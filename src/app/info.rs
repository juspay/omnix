//! Nix info UI

use std::fmt::Display;

use dioxus::prelude::*;
use nix_rs::{config::NixConfig, env::NixEnv, info::NixInfo, version::NixVersion};

use crate::{app::state::AppState, app::widget::Loader};

/// Nix information
#[component]
pub fn Info() -> Element {
    let title = "Nix Info";
    let state = AppState::use_state();
    let nix_info = state.nix_info.read();
    rsx! {
        h1 { class: "text-5xl font-bold", title }
        if nix_info.is_loading_or_refreshing() {
            Loader {}
        }
        div { class: "flex items-center justify-center",
            { nix_info.render_with(|v| rsx! { NixInfoView { info: v.clone() } }) }
        }
    }
}

#[component]
fn NixInfoView(info: NixInfo) -> Element {
    rsx! {
        div { class: "flex flex-col max-w-prose p-4 space-y-8 bg-white border-2 rounded border-base-400",
            div {
                b { "Nix Version" }
                div { class: "p-1 my-1 rounded bg-primary-50", NixVersionView { version: info.nix_version } }
            }
            div {
                b { "Nix Config" }
                NixConfigView { config: info.nix_config.clone() }
            }
            div {
                b { "Nix Env" }
                NixEnvView { env: info.nix_env.clone() }
            }
        }
    }
}

#[component]
fn NixVersionView(version: NixVersion) -> Element {
    rsx! {a { href: nix_rs::refs::RELEASE_HISTORY, class: "font-mono hover:underline", target: "_blank", "{version}" }}
}

#[component]
fn NixConfigView(config: NixConfig) -> Element {
    rsx! {
        div { class: "py-1 my-1 rounded bg-primary-50",
            table { class: "text-right",
                tbody {
                    TableRow { name: "Local System", title: config.system.description, "{config.system.value}" }
                    TableRow { name: "Max Jobs", title: config.max_jobs.description, "{config.max_jobs.value}" }
                    TableRow { name: "Cores per build", title: config.cores.description, "{config.cores.value}" }
                    TableRow { name: "Nix Caches", title: config.substituters.description, ConfigValList { items: config.substituters.value } }
                }
            }
        }
    }
}

#[component]
fn ConfigValList<T>(items: Vec<T>) -> Element
where
    T: Display,
{
    rsx! {
        div { class: "flex flex-col space-y-4",
            for item in items {
                li { class: "list-disc", "{item}" }
            }
        }
    }
}

#[component]
fn NixEnvView(env: NixEnv) -> Element {
    rsx! {
        div { class: "py-1 my-1 rounded bg-primary-50",
            table { class: "text-right",
                tbody {
                    TableRow { name: "Current User", title: "Logged-in user", code { "{env.current_user}" } }
                    TableRow { name: "OS", title: "Operating System", code { "{env.os}" } }
                    TableRow { name: "Total disk space", title: "Total disk space on the current machine", code { "{env.total_disk_space}" } }
                    TableRow { name: "Total RAM", title: "Total memory on the current machine", code { "{env.total_memory}" } }
                }
            }
        }
    }
}

#[component]
fn TableRow(name: &'static str, title: String, children: Element) -> Element {
    rsx! {
        tr { title: "{title}",
            td { class: "px-4 py-2 font-semibold text-base-700", "{name}" }
            td { class: "px-4 py-2 text-left",
                code { { children } }
            }
        }
    }
}
