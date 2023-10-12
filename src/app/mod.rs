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
use nix_rs::flake::url::FlakeUrl;

use crate::app::{
    flake::{Flake, FlakeRaw},
    health::Health,
    info::Info,
    state::AppState,
    widget::Loader,
};

#[derive(Routable, PartialEq, Debug, Clone)]
#[rustfmt::skip]
enum Route {
    #[layout(Wrapper)]
        #[route("/")]
        Dashboard {},
        #[route("/about")]
        About {},
        #[route("/flake")]
        Flake {},
        #[route("/flake/raw")]
        FlakeRaw {},
        #[route("/health")]
        Health {},
        #[route("/info")]
        Info {},
}

fn Wrapper(cx: Scope) -> Element {
    render! {
        Nav {}
        Outlet::<Route> {}
        footer { class: "flex flex-row justify-center w-full p-4", img { src: "images/128x128.png", width: "32", height: "32" } }
    }
}

/// Main frontend application container
pub fn App(cx: Scope) -> Element {
    AppState::provide_state(cx);
    use_shared_state_provider(cx, || {
        FlakeUrl::suggestions()
            .first()
            .map(Clone::clone)
            .unwrap_or_default()
    });
    render! {
        body {
            // Can't do this, because Tauri window has its own scrollbar. :-/
            // class: "overflow-y-scroll",
            div { class: "flex justify-center w-full min-h-screen bg-center bg-cover bg-base-200",
                div { class: "flex flex-col items-stretch mx-auto sm:container sm:max-w-screen-md",
                    main { class: "flex flex-col px-2 mb-8 space-y-3 text-center", Router::<Route> {} }
                }
            }
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
        h1 { class: "text-5xl font-bold", "Dashboard" }
        div { id: "cards", class: "flex flex-row flex-wrap",
            Card { href: Route::Health {},
                "Nix Health Check "
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
            Card { href: Route::Info {}, "Nix Info ℹ️" }
            Card { href: Route::Flake {}, "Flake Dashboard ❄️️" }
        }
    }
}

/// Navigation bar
///
/// TODO Switch to breadcrumbs, as it simplifes the design overall.
fn Nav(cx: Scope) -> Element {
    let class = "px-3 py-2";
    let active_class = "bg-white text-primary-800 font-bold";
    render! {
        nav { class: "flex flex-row w-full mb-8 text-white md:rounded-b bg-primary-800",
            Link { to: Route::Dashboard {}, class: class, active_class: active_class, "Dashboard" }
            Link { to: Route::Flake {}, class: class, active_class: active_class, "Flake" }
            Link { to: Route::Health {}, class: class, active_class: active_class, "Nix Health" }
            Link { to: Route::Info {}, class: class, active_class: active_class, "Nix Info" }
            Link { to: Route::About {}, class: class, active_class: active_class, "About" }
            div { class: "flex-grow font-bold text-end {class}", "🌍 nix-browser" }
        }
    }
}

/// About page
fn About(cx: Scope) -> Element {
    render! {
        h1 { class: "text-5xl font-bold", "About" }
        p {
            "nix-browser is still work in progress. Track its development "
            a {
                href: "https://github.com/juspay/nix-browser",
                class: "underline text-primary-500 hover:no-underline",
                rel: "external",
                target: "_blank",
                "on Github"
            }
        }
    }
}
