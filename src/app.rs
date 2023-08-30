//! Frontend UI entry point

use leptos::*;
use leptos_meta::*;
use leptos_query::*;
use leptos_router::*;

use crate::widget::*;
use leptos_extra::{
    query::{self, RefetchQueryButton},
    signal::{provide_signal, use_signal, SignalWithResult},
};
use nix_rs::nix::command::Refresh;
use nix_rs::nix::{
    flake::{get_flake, url::FlakeUrl},
    health::{get_nix_health, traits::Check},
    info::get_nix_info,
};

/// Main frontend application container
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
                            <Route path="/flake" view=NixFlake>
                                <Route path="" view=NixFlakeHome/>
                                <Route path="raw" view=NixFlakeRaw/>
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
            <A exact=false href="/flake" class=class>
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
                class="flex items-center justify-center w-48 h-48 p-2 m-2 border-2 rounded-lg shadow border-base-400 active:shadow-none bg-base-100 hover:bg-primary-200"
            >
                <span class="text-3xl text-base-800">{children(cx)}</span>
            </A>
        }
    }
    view! { cx,
        <Title text="Dashboard"/>
        <h1 class="text-5xl font-bold">"Dashboard"</h1>
        <div id="cards" class="flex flex-row flex-wrap">
            <SuspenseWithErrorHandling>
                <Card href="/health">"Nix Health Check " {report}</Card>
            </SuspenseWithErrorHandling>
            <Card href="/info">"Nix Info ℹ️"</Card>
            <Card href="/flake">"Flake Overview ❄️️"</Card>
        </div>
    }
}

/// Nix flake dashboard
#[component]
fn NixFlake(cx: Scope) -> impl IntoView {
    let suggestions = FlakeUrl::suggestions();
    let url = use_signal::<FlakeUrl>(cx);
    let refresh = use_signal::<Refresh>(cx);
    let query = move || (url(), refresh());
    let result = query::use_server_query(cx, query, get_flake);
    view! { cx,
        <Title text="Nix Flake"/>
        <h1 class="text-5xl font-bold">{"Nix Flake"}</h1>
        <TextInput id="nix-flake-input" label="Load a Nix Flake" val=url suggestions/>
        <RefetchQueryButton result query/>
        <Outlet/>
    }
}

#[component]
fn NixFlakeHome(cx: Scope) -> impl IntoView {
    let url = use_signal::<FlakeUrl>(cx);
    let refresh = use_signal::<Refresh>(cx);
    let query = move || (url(), refresh());
    let result = query::use_server_query(cx, query, get_flake);
    let data = result.data;
    view! { cx,
        <div class="p-2 my-1">
            <SuspenseWithErrorHandling>{data}</SuspenseWithErrorHandling>
        </div>
    }
}

#[component]
fn NixFlakeRaw(cx: Scope) -> impl IntoView {
    let url = use_signal::<FlakeUrl>(cx);
    let refresh = use_signal::<Refresh>(cx);
    let query = move || (url(), refresh());
    let result = query::use_server_query(cx, query, get_flake);
    let data = result.data;
    view! { cx,
        <div>
            <A href="/flake">"< Back"</A>
        </div>
        <div class="px-4 py-2 font-mono text-xs text-left text-gray-500 border-2 border-black">
            <SuspenseWithErrorHandling>
                {move || { data.get().map(|r| r.map(|v| v.output.into_view(cx))) }}
            </SuspenseWithErrorHandling>
        </div>
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
