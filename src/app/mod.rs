//! Frontend UI entry point

// Workaround for https://github.com/rust-lang/rust-analyzer/issues/15344
#![allow(non_snake_case)]

mod flake;
mod health;
mod info;
mod state;

use dioxus::prelude::*;
use dioxus_router::prelude::*;
use nix_rs::flake::url::FlakeUrl;

use crate::app::{flake::Flake, health::Health, info::Info, state::AppState};

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
    use_context_provider(cx, AppState::default);
    use_shared_state_provider(cx, || {
        FlakeUrl::suggestions()
            .first()
            .map(Clone::clone)
            .unwrap_or_default()
    });
    // TODO: per-route title
    // This should also be in desktop window title bar.
    render! {
        body {
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
    use_future(cx, (), |_| async move {
        state.update_health_checks().await;
    });
    let health_checks = &*state.health_checks.read();
    // A Card component
    #[component]
    fn Card<'a>(cx: Scope, href: &'static str, children: Element<'a>) -> Element<'a> {
        render! {
            // TODO: Use Link
            a {
                href: "{href}",
                class: "flex items-center justify-center w-48 h-48 p-2 m-2 border-2 rounded-lg shadow border-base-400 active:shadow-none bg-base-100 hover:bg-primary-200",
                span { class: "text-3xl text-base-800", children }
            }
        }
    }
    render! {
        h1 { class: "text-5xl font-bold", "Dashboard" }
        div { id: "cards", class: "flex flex-row flex-wrap",
            Card { href: "/health",
                "Nix Health Check "
                match health_checks {
                    Some(Ok(checks)) => {
                        if checks.iter().all(|check| check.result.green()) {
                            "✅"
                        } else {
                            "❌"
                        }
                    },
                    // TODO: Error handling in dioxus?
                    Some(Err(_)) => "?",
                    None => "⏳",
                }
            }
            Card { href: "/info", "Nix Info ℹ️" }
            Card { href: "/flake", "Flake Overview ❄️️" }
        }
    }
}

/// Navigation bar
///
/// TODO Switch to breadcrumbs, as it simplifes the design overall.
fn Nav(cx: Scope) -> Element {
    let class = "px-3 py-2";
    render! {
        // TODO: active/inactive styling
        nav { class: "flex flex-row w-full mb-8 text-white md:rounded-b bg-primary-800",
            Link { to: Route::Dashboard {}, class: class, "Dashboard" }
            Link { to: Route::Flake {}, class: class, "Flake" }
            Link { to: Route::Health {}, class: class, "Nix Health" }
            Link { to: Route::Info {}, class: class, "Nix Info" }
            Link { to: Route::About {}, class: class, "About" }
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

/*
#[component]
pub fn App(cx: Scope) -> impl IntoView {
    provide_meta_context(cx);
    provide_query_client(cx);
    provide_signal::<FlakeUrl>(
        cx,
        FlakeUrl::suggestions()
            .first()
            .map(Clone::clone)
            .unwrap_or_default(),
    );
    provide_signal::<Refresh>(cx, false.into()); // refresh flag is unused, but we may add it to UI later.

    view! { cx,
        <Stylesheet id="leptos" href="/pkg/nix-browser.css"/>
        <Title formatter=|s| format!("{s} ― nix-browser")/>
        <Router fallback=|cx| {
            view! { cx, <NotFound/> }
        }>
            <Body class="overflow-y-scroll"/>
            <div class="flex justify-center w-full min-h-screen bg-center bg-cover bg-base-200">
                <div class="flex flex-col items-stretch mx-auto sm:container sm:max-w-screen-md">
                    <Nav/>
                    <main class="flex flex-col px-2 mb-8 space-y-3 text-center">
                        <Routes>
                            <Route path="" view=Dashboard/>
                            <Route path="/flake" view=NixFlakeRoute>
                                <Route path="" view=NixFlakeHomeRoute/>
                                <Route path="raw" view=NixFlakeRawRoute/>
                            </Route>
                            <Route path="/health" view=NixHealthRoute/>
                            <Route path="/info" view=NixInfoRoute/>
                            <Route path="/about" view=About/>
                        </Routes>
                    </main>
                </div>
            </div>
        </Router>
    }
}
*/
