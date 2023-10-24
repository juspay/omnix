//! Frontend UI entry point

// Workaround for https://github.com/rust-lang/rust-analyzer/issues/15344
#![allow(non_snake_case)]

mod flake;
mod health;
mod info;
mod state;
mod widget;

use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::app::{
    flake::{Flake, FlakeRaw},
    health::Health,
    info::Info,
    state::AppState,
    widget::{Loader, Scrollable},
};

#[derive(Routable, PartialEq, Debug, Clone)]
#[rustfmt::skip]
enum Route {
    #[layout(Wrapper)]
        #[route("/")]
        Dashboard {},
        #[route("/flake")]
        Flake {},
        #[route("/flake/raw")]
        FlakeRaw {},
        #[route("/health")]
        Health {},
        #[route("/info")]
        Info {},
}

/// Main frontend application container
pub fn App(cx: Scope) -> Element {
    AppState::provide_state(cx);
    render! {
        body { class: "bg-base-100", Router::<Route> {} }
    }
}

fn Wrapper(cx: Scope) -> Element {
    render! {
        div { class: "flex flex-col text-center justify-between w-full min-h-screen",
            div {
                TopBar {}
                div { class: "m-2 py-2", Outlet::<Route> {} }
            }
            Footer {}
        }
    }
}

#[component]
fn TopBar(cx: Scope) -> Element {
    let state = AppState::use_state(cx);
    let health_checks = state.health_checks.read();
    let nix_info = state.nix_info.read();
    render! {
        div { class: "flex justify-between items-center w-full p-2 bg-base-200 sticky top-0",
            div { class: "flex space-x-2",
                Link { to: Route::Dashboard {}, "🏠" }
                Link { to: Route::Health {},
                    span { title: "Nix Health Status",
                        match (*health_checks).current_value() {
                            Some(Ok(checks)) => render! {
                                if checks.iter().all(|check| check.result.green()) {
                                    "✅"
                                } else {
                                    "❌"
                                }
                            },
                            Some(Err(err)) => render! { "{err}" },
                            None => render! { Loader {} },
                        }
                    }
                }
            }
            Link { to: Route::Info {},
                span {
                    "ℹ️ Nix "
                    match (*nix_info).current_value() {
                        Some(Ok(info)) => render! {
                            "{info.nix_version} on {info.nix_env.os}"
                        },
                        Some(Err(err)) => render! { "{err}" },
                        None => render! { Loader {} },
                    }
                }
            }
        }
    }
}

#[component]
fn Footer(cx: Scope) -> Element {
    render! {
        footer { class: "flex flex-row justify-center w-full p-2",
            a { href: "https://github.com/juspay/nix-browser", img { src: "images/128x128.png", class: "h-6" } }
        }
    }
}

// Home page
fn Dashboard(cx: Scope) -> Element {
    tracing::debug!("Rendering Dashboard page");
    render! {
        div {
            id: "cards",
            class: "flex flex-row justify-center items-center text-3xl flex-wrap",
            // TODO: This will contain the flake search bar directly (like
            // Google), along with a list of recently visited flakes (like
            // VSCode home tab)
            Link { to: Route::Flake {}, class: "underline hover:no-underline", "Browse flakes" }
        }
    }
}
