//! Nix health check UI

use dioxus::prelude::*;
use nix_health::traits::{Check, CheckResult};

use crate::{
    app::state::AppState,
    widget::{Loader, RefreshButton},
};

/// Nix health checks
pub fn Health(cx: Scope) -> Element {
    let state = AppState::use_state(cx);
    let health_checks = state.health_checks.read();
    let title = "Nix Health";
    let busy = (*health_checks).is_loading_or_refreshing();
    let click_event = move |_event| {
        cx.spawn(async move {
            state.update_health_checks().await;
        });
    };
    render! {
        h1 { class: "text-5xl font-bold", title }
        RefreshButton { busy: busy, handler: click_event }
        div { class: "my-1",
            match (*health_checks).current_value() {
                None => render! { Loader {}},
                Some(Ok(checks)) => render! {
                div { class: "flex flex-col items-stretch justify-start space-y-8 text-left",
                    for check in checks {
                        ViewCheck { check: check.clone() }
                    }
                }
                },
                Some(Err(err)) => render! { "{err}" }
            }
        }
    }
}

#[component]
fn ViewCheck(cx: Scope, check: Check) -> Element {
    render! {
        div { class: "contents",
            details {
                open: check.result != CheckResult::Green,
                class: "my-2 bg-white border-2 rounded-lg cursor-pointer hover:bg-primary-100 border-base-300",
                summary { class: "p-4 text-xl font-bold",
                    CheckResultSummaryView { green: check.result.green() }
                    " "
                    check.title.clone()
                }
                div { class: "p-4",
                    div { class: "p-2 my-2 font-mono text-sm bg-black text-base-100", check.info.clone() }
                    div { class: "flex flex-col justify-start space-y-4",
                        match check.result.clone() {
                            CheckResult::Green => render! { "" },
                            CheckResult::Red { msg, suggestion } => render! {
                                h3 { class: "my-2 font-bold text-l" }
                                div { class: "p-2 bg-red-400 rounded bg-border", msg }
                                h3 { class: "my-2 font-bold text-l" }
                                div { class: "p-2 bg-blue-400 rounded bg-border", suggestion }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn CheckResultSummaryView(cx: Scope, green: bool) -> Element {
    if *green {
        render! { span { class: "text-green-500", "✓" } }
    } else {
        render! { span { class: "text-red-500", "✗" } }
    }
}
