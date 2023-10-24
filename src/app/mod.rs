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
        body { Router::<Route> {} }
    }
}

fn Wrapper(cx: Scope) -> Element {
    render! {
        div { class: "flex flex-col text-center justify-between w-full overflow-hidden h-screen bg-base-100",
            div {
                TopBar {}
                Nav {}
                Scrollable {
                    div { class: "m-2 py-2", Outlet::<Route> {} }
                }
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
        div { class: "flex justify-between items-center w-full p-2",
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
        footer { class: "flex flex-row justify-center w-full p-2 bg-primary-100",
            a { href: "https://github.com/juspay/nix-browser", img { src: "images/128x128.png", class: "h-6" } }
        }
    }
}

// Home page
fn Dashboard(cx: Scope) -> Element {
    tracing::debug!("Rendering Dashboard page");
    let state = AppState::use_state(cx);
    let health_checks = state.health_checks.read();
    // A Card component
    #[component]
    fn Card<'a>(cx: Scope, href: Route, children: Element<'a>) -> Element<'a> {
        render! {
            Link {
                to: "{href}",
                class: "flex items-center justify-center w-48 h-48 p-2 m-2 border-2 rounded-lg shadow border-base-400 active:shadow-none bg-base-100 hover:bg-primary-200",
                span { class: "text-3xl text-base-800", children }
            }
        }
    }
    render! {
        div {
            id: "cards",
            class: "flex flex-row justify-center items-center flex-wrap",
            Card { href: Route::Health {},
                "Health "
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
            Card { href: Route::Info {}, "Info ℹ️" }
            Card { href: Route::Flake {}, "Flake ❄️️" }
        }
    }
}

/// Navigation bar
///
/// TODO Switch to breadcrumbs, as it simplifes the design overall.
fn Nav(cx: Scope) -> Element {
    // Common class for all tabs
    let class = "flex-grow block py-1.5 mx-1 text-center rounded-t-md";

    // Active tab styling: Highlighted background and pronounced text color
    let active_class = "bg-primary-200 font-bold text-black";

    // Inactive tab styling: Muted background and text color
    let inactive_class = "bg-gray-200 text-gray-600";

    render! {
        nav { class: "flex flex-row w-full bg-gray-100 border-b border-gray-300 pt-2",

            Link {
                to: Route::Dashboard {},
                class: "{class} {inactive_class}",
                active_class: active_class,
                "Dashboard"
            }
            Link {
                to: Route::Flake {},
                class: "{class} {inactive_class}",
                active_class: active_class,
                "Flake"
            }
            Link {
                to: Route::Health {},
                class: "{class} {inactive_class}",
                active_class: active_class,
                "Nix Health"
            }
            Link {
                to: Route::Info {},
                class: "{class} {inactive_class}",
                active_class: active_class,
                "Nix Info"
            }
            div { class: "flex-grow font-bold text-end px-3 py-1",
                a {
                    href: "https://github.com/juspay/nix-browser",
                    class: "text-primary-700",
                    rel: "external",
                    target: "_blank",
                    "nix-browser"
                }
            }
        }
    }
}
