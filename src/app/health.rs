//! Nix health check UI

use dioxus::prelude::*;
use nix_health::traits::{Check, CheckResult};

use crate::{app::state::AppState, app::widget::Loader};

/// Nix health checks
pub fn Health() -> Element {
    let state = AppState::use_state();
    let health_checks = state.health_checks.read();
    let title = "Nix Health";
    rsx! {
        h1 { class: "text-5xl font-bold", title: title }
        if health_checks.is_loading_or_refreshing() {
            Loader {}
        }
        { health_checks.render_with(|checks| rsx! {
            div { class: "flex flex-col items-stretch justify-start space-y-8 text-left",
                for check in checks {
                        ViewCheck { check: check.clone() }
                }
            }
        }) }
    }
}

#[component]
fn ViewCheck(check: Check) -> Element {
    rsx! {
        div { class: "contents",
            details {
                open: check.result != CheckResult::Green,
                class: "my-2 bg-white border-2 rounded-lg cursor-pointer hover:bg-primary-100 border-base-300",
                summary { class: "p-4 text-xl font-bold",
                    CheckResultSummaryView { green: check.result.green() }
                    " "
                    { check.title.clone() }
                }
                div { class: "p-4",
                    div { class: "p-2 my-2 font-mono text-sm bg-black text-base-100",
                        { check.info.clone() }
                    }
                    div { class: "flex flex-col justify-start space-y-4",
                        match check.result.clone() {
                            CheckResult::Green => rsx! { "" },
                            CheckResult::Red { msg, suggestion } => rsx! {
                                h3 { class: "my-2 font-bold text-l" }
                                div { class: "p-2 bg-red-400 rounded bg-border", { msg } }
                                h3 { class: "my-2 font-bold text-l" }
                                div { class: "p-2 bg-blue-400 rounded bg-border", { suggestion } }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn CheckResultSummaryView(green: bool) -> Element {
    if green {
        rsx! { span { class: "text-green-500", "✓" } }
    } else {
        rsx! { span { class: "text-red-500", "✗" } }
    }
}
