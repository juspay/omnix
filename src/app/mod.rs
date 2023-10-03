//! Frontend UI entry point

// Workaround for https://github.com/rust-lang/rust-analyzer/issues/15344
#![allow(non_snake_case)]

mod flake;
mod health;
mod info;

use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::widget::*;

#[derive(Routable, PartialEq, Debug, Clone)]
enum Route {
    #[route("/")]
    Dashboard {},
    #[route("/about")]
    About {},
}

/// Main frontend application container
pub fn App(cx: Scope) -> Element {
    // TODO: per-route title
    render! {
        body {
            div { class: "flex justify-center w-full min-h-screen bg-center bg-cover bg-base-200",
                div { class: "flex flex-col items-stretch mx-auto sm:container sm:max-w-screen-md",
                    main { class: "flex flex-col px-2 mb-8 space-y-3 text-center",
                        Nav {}
                        p { "It is WIP" }
                        ul { li { "Tailwind works" } }
                        img { src: "images/128x128.png" }
                    }
                }
            }
        }
    }
}

// Home page
fn Dashboard(cx: Scope) -> Element {
    tracing::debug!("Rendering Dashboard page");
    /* let result = query::use_server_query(cx, || (), get_nix_health);
    let data = result.data;
    let healthy = Signal::derive(cx, move || {
        data.with_result(|checks| checks.iter().all(|check| check.result.green()))
    });
    */
    // A Card component
    #[inline_props]
    fn Card<'a>(cx: Scope, href: &'static str, children: Element<'a>) -> Element<'a> {
        render! {
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
        nav { class: "flex flex-row w-full mb-8 text-white md:rounded-b bg-primary-800",
            a { href: "/", class: class, "Dashboard" }
            a { href: "/flake", class: class, "Flake" }
            a { href: "/health", class: class, "Nix Health" }
            a { href: "/info", class: class, "Nix Info" }
            a { href: "/about", class: class, "About" }
            div { class: "flex-grow font-bold text-end {class}", "🌍 nix-browser" }
        }
    }
}

/// About page
fn About(cx: Scope) -> Element {
    render! {
        h1 { "About" }
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
