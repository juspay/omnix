//! Frontend UI entry point

use leptos::*;
use leptos_meta::*;
use leptos_query::*;
use leptos_router::*;

use crate::leptos_extra::{
    query::{self, QueryInput, RefetchQueryButton},
    signal::{provide_signal, use_signal, OptionResult, SignalWithResult},
};
use crate::nix::{
    flake::{get_flake, url::FlakeUrl},
    health::{get_nix_health, traits::Check},
    info::get_nix_info,
};
use crate::widget::*;

/// Main frontend application container
#[component]
pub fn App(cx: Scope) -> impl IntoView {
    provide_meta_context(cx);
    provide_query_client(cx);
    provide_signal::<FlakeUrl>(cx, "github:srid/haskell-template".into());

    view! { cx,
        <Stylesheet id="leptos" href="/pkg/nix-browser.css"/>
        <Title formatter=|s| format!("{s} ― nix-browser")/>
        <Router fallback=|cx| {
            view! { cx, <NotFound/> }
        }>
            <Body class="overflow-y-scroll"/>
            <div class="flex justify-center w-full min-h-screen bg-center bg-cover bg-base-200">
                <div class="container flex flex-col items-stretch mx-auto max-w-prose">
                    <Nav/>
                    <main class="flex flex-col px-2 space-y-3 text-center">
                        <Routes>
                            <Route path="" view=Dashboard/>
                            <Route path="/flake" view=NixFlake>
                                <Route path="" view=NixFlakeHome/>
                                <Route path=":system" view=NixFlakePerSystem/>
                            </Route>
                            <Route path="/health" view=NixHealth/>
                            <Route path="/info" view=NixInfo/>
                            <Route path="/about" view=About/>
                        </Routes>
                    </main>
                </div>
            </div>
        </Router>
    }
}

/// Navigation bar
///
/// TODO Switch to breadcrumbs, as it simplifes the design overall.
#[component]
fn Nav(cx: Scope) -> impl IntoView {
    let class = "px-3 py-2";
    view! { cx,
        <nav class="flex flex-row w-full mb-8 text-white md:rounded-b bg-primary-800">
            <A exact=true href="/" class=class>
                "Dashboard"
            </A>
            <A exact=true href="/flake" class=class>
                "Flake"
            </A>
            <A exact=true href="/health" class=class>
                "Nix Health"
            </A>
            <A exact=true href="/info" class=class>
                "Nix Info"
            </A>
            <A exact=true href="/about" class=class>
                "About"
            </A>
            <div class=format!("flex-grow font-bold text-end {}", class)>"🌍 nix-browser"</div>
        </nav>
    }
}

/// Home page
#[component]
fn Dashboard(cx: Scope) -> impl IntoView {
    tracing::debug!("Rendering Dashboard page");
    let result = query::use_server_query(cx, || (), get_nix_health);
    let data = result.data;
    let report = move || data.with_result(|v| v.report());
    // A Card component
    #[component]
    fn Card(cx: Scope, href: &'static str, children: Children) -> impl IntoView {
        view! { cx,
            <A
                href=href
                class="flex items-center justify-center w-64 h-48 p-2 m-2 border-2 rounded-lg shadow border-base-400 active:shadow-none bg-base-100 hover:bg-primary-200"
            >
                <span class="text-4xl text-base-800">{children(cx)}</span>
            </A>
        }
    }
    view! { cx,
        <Title text="Dashboard"/>
        <h1 class="text-5xl font-bold">"Dashboard"</h1>
        <div id="cards" class="flex flex-row">
            <SuspenseWithErrorHandling>
                <Card href="/health">"Nix Health Check " {report}</Card>
            </SuspenseWithErrorHandling>
            <Card href="/info">"Nix Info ℹ️"</Card>
        </div>
    }
}

/// Nix flake dashboard
#[component]
fn NixFlake(cx: Scope) -> impl IntoView {
    let suggestions = FlakeUrl::suggestions();
    let (query, set_query) = use_signal::<FlakeUrl>(cx);
    let result = query::use_server_query(cx, query, get_flake);
    view! { cx,
        <Title text="Nix Flake"/>
        <h1 class="text-5xl font-bold">{"Nix Flake"}</h1>
        <QueryInput id="nix-flake-input" query set_query suggestions/>
        <RefetchQueryButton result query/>

        // FIXME: putting this here causes route switch bugs
        // github.com/leptos-rs/leptos/issues/1569
        // <NixFlakeNav />

        <Outlet/>
    }
}

#[component]
fn NixFlakeNav(cx: Scope) -> impl IntoView {
    let (query, _) = use_signal::<FlakeUrl>(cx);
    let result = query::use_server_query(cx, query, get_flake);
    let data = result.data;
    view! { cx,
        <ul class="my-2">
            // TODO: Cleanly do navigation
            <li>
                <a href="/flake">"Main"</a>
            </li>
            <SuspenseWithErrorHandling>
                {move || {
                    data.get()
                        .map_option_result(move |v| {
                            v.per_system
                                .0
                                .keys()
                                .clone()
                                .map(|k| {
                                    {
                                        let system = &k.to_string().clone();

                                        view! { cx,
                                            <li>
                                                <a
                                                    class="hover:bg-primary-200"
                                                    href=format!("/flake/{}", system)
                                                >
                                                    {system}
                                                </a>
                                            </li>
                                        }
                                    }
                                })
                                .collect_view(cx)
                        })
                }}

            </SuspenseWithErrorHandling>

        </ul>
    }
}

#[component]
fn NixFlakeHome(cx: Scope) -> impl IntoView {
    let (query, _) = use_signal::<FlakeUrl>(cx);
    let result = query::use_server_query(cx, query, get_flake);
    let data = result.data;
    view! { cx,
        <NixFlakeNav/>
        <div class="p-2 my-1 text-left border-2 border-black">
            <SuspenseWithErrorHandling>{data}</SuspenseWithErrorHandling>
        </div>
    }
}

/// Nix flake dashboard
#[component]
fn NixFlakePerSystem(cx: Scope) -> impl IntoView {
    let params = use_params_map(cx);
    let system = move || params.with(|params| params.get("system").cloned().unwrap_or_default());

    let (query, _) = use_signal::<FlakeUrl>(cx);
    let result = query::use_server_query(cx, query, get_flake);
    let data = result.data;
    let data = move || data.with_result(move |v| v.per_system.0[&system().into()].clone());
    view! { cx,
        <NixFlakeNav/>
        <SuspenseWithErrorHandling>{data}</SuspenseWithErrorHandling>
    }
}

/// Nix information
#[component]
fn NixInfo(cx: Scope) -> impl IntoView {
    let title = "Nix Info";
    let result = query::use_server_query(cx, || (), get_nix_info);
    let data = result.data;
    view! { cx,
        <Title text=title/>
        <h1 class="text-5xl font-bold">{title}</h1>
        <RefetchQueryButton result query=|| ()/>
        <div class="my-1 text-left">
            <SuspenseWithErrorHandling>{data}</SuspenseWithErrorHandling>
        </div>
    }
}

/// Nix health checks
#[component]
fn NixHealth(cx: Scope) -> impl IntoView {
    let title = "Nix Health";
    let result = query::use_server_query(cx, || (), get_nix_health);
    let data = result.data;
    view! { cx,
        <Title text=title/>
        <h1 class="text-5xl font-bold">{title}</h1>
        <RefetchQueryButton result query=|| ()/>
        <div class="my-1">
            <SuspenseWithErrorHandling>{data}</SuspenseWithErrorHandling>
        </div>
    }
}

/// About page
#[component]
fn About(cx: Scope) -> impl IntoView {
    view! { cx,
        <Title text="About"/>
        <h1 class="text-5xl font-bold">"About"</h1>
        <p>
            "nix-browser is still work in progress. Track its development "
            <LinkExternal link="https://github.com/juspay/nix-browser" text="on Github"/>
        </p>
    }
}
