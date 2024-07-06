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
    widget::{Loader, RefreshButton},
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
pub fn App() -> Element {
    AppState::provide_state();
    rsx! {
        body { class: "bg-base-100 overflow-hidden", Router::<Route> {} }
    }
}

fn Wrapper() -> Element {
    rsx! {
        div { class: "flex flex-col text-center justify-between w-full h-screen",
            TopBar {}
            div { class: "m-2 py-2 overflow-auto", Outlet::<Route> {} }
            Footer {}
        }
    }
}

#[component]
fn TopBar() -> Element {
    let state = AppState::use_state();
    let health_checks = state.health_checks.read();
    let nix_info = state.nix_info.read();
    rsx! {
        div { class: "flex justify-between items-center w-full p-2 bg-primary-100 shadow",
            div { class: "flex space-x-2",
                Link { to: Route::Dashboard {}, "🏠" }
            }
            div { class: "flex space-x-2",
                ViewRefreshButton {}
                Link { to: Route::Health {},
                    span { title: "Nix Health Status",
                        match (*health_checks).current_value() {
                            Some(Ok(checks)) => rsx! {
                                if checks.iter().all(|check| check.result.green()) {
                                    "✅"
                                } else {
                                    "❌"
                                }
                            },
                            Some(Err(err)) => rsx! { "{err}" },
                            None => rsx! { Loader {} },
                        }
                    }
                }
                Link { to: Route::Info {},
                    span {
                        "Nix "
                        match (*nix_info).current_value() {
                            Some(Ok(info)) => rsx! {
                                "{info.nix_version} on {info.nix_env.os}"
                            },
                            Some(Err(err)) => rsx! { "{err}" },
                            None => rsx! { Loader {} },
                        }
                    }
                }
            }
        }
    }
}

/// Intended to refresh the data behind the current route.
#[component]
fn ViewRefreshButton() -> Element {
    let state = AppState::use_state();
    let (busy, mut refresh_signal) = match use_route() {
        Route::Flake {} => Some((
            state.flake.read().is_loading_or_refreshing(),
            state.flake_refresh,
        )),
        Route::Health {} => Some((
            state.health_checks.read().is_loading_or_refreshing(),
            state.health_checks_refresh,
        )),
        Route::Info {} => Some((
            state.nix_info.read().is_loading_or_refreshing(),
            state.nix_info_refresh,
        )),
        _ => None,
    }?;
    rsx! {
        { RefreshButton (
            busy,
            move |_| {
                refresh_signal.write().request_refresh();
            }
        ) }
    }
}

#[component]
fn Footer() -> Element {
    rsx! {
        footer { class: "flex flex-row justify-center w-full bg-primary-100 p-2",
            a { href: "https://github.com/juspay/nix-browser", img { src: "images/128x128.png", class: "h-4" } }
        }
    }
}

// Home page
fn Dashboard() -> Element {
    tracing::debug!("Rendering Dashboard page");
    let state = AppState::use_state();
    rsx! {
        div { class: "pl-4",
            h2 { class: "text-2xl", "Enter a flake URL:" }
            // TODO: search input here
            p { "TODO: search input" }
            h2 { class: "text-2xl", "Or, try one of these:" }
            div { class: "flex flex-col",
                for flake_url in state.flake_cache.read().recent_flakes() {
                    a {
                        onclick: move |_| {
                            let mut state = AppState::use_state();
                            let nav = use_navigator();
                            state.set_flake_url(flake_url.clone());
                            nav.replace(Route::Flake {});
                        },
                        class: "cursor-pointer text-primary-600 underline hover:no-underline",
                        "{flake_url.clone()}"
                    }
                }
            }
        }
    }
}
