//! Frontend UI entry point

use std::fmt::Display;
use std::hash::Hash;
use std::marker::PhantomData;
use std::str::FromStr;

use leptos::*;
use leptos_meta::*;
use leptos_query::*;
use leptos_router::*;
use tracing::instrument;

use crate::nix::flake::GetNixFlake;
use crate::nix::health::traits::Check;
use crate::query::{self, RefetchQueryButton, ServerQueryVal};
use crate::widget::*;

fn provide_signal<T: 'static>(cx: Scope, default: T) {
    let (get, set) = create_signal(cx, default);
    provide_context(cx, (get, set));
}

#[instrument(name = "use_signal")]
fn use_signal<T>(cx: Scope) -> (ReadSignal<T>, WriteSignal<T>) {
    use_context(cx)
        .ok_or_else(|| {
            // This happens if the dev forgets to call `provide_signal::<T>` in
            // the parent scope
            let msg = format!(
                "no signal provided for type: {}",
                std::any::type_name::<T>()
            );
            tracing::error!(msg);
            msg
        })
        .unwrap()
}

/// Main frontend application container
#[component]
pub fn App(cx: Scope) -> impl IntoView {
    provide_meta_context(cx);
    provide_query_client(cx);
    provide_signal::<GetNixFlake>(
        cx,
        GetNixFlake {
            url: "github:srid/haskell-template".into(),
        },
    );

    view! { cx,
        <Stylesheet id="leptos" href="/pkg/nix-browser.css"/>
        <Router fallback=|cx| {
            view! { cx, <NotFound/> }
        }>
            <Title formatter=|s| format!("{s} - nix-browser")/>
            <div class="flex justify-center w-full min-h-screen bg-center bg-cover bg-base-200">
                <div class="container flex flex-col items-center mx-auto max-w-prose">
                    <Nav/>
                    <div class="z-0 flex col-start-1 row-start-1 px-2 text-center">
                        <div class="flex flex-col space-y-3">
                            <Routes>
                                <Route path="" view=Dashboard/>
                                <Route path="/flake" view=NixFlake/>
                                <Route path="/health" view=NixHealth/>
                                <Route path="/info" view=NixInfo/>
                                <Route path="/about" view=About/>
                            </Routes>
                        </div>
                    </div>
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
    let res = query::use_nix_health_query(cx);
    let report = Signal::derive(cx, move || res.data.get().map(|r| r.map(|v| v.report())));
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

/// An input element tied to a [leptos_use::Query]
///
/// TODO: remove hardcoded stuff
#[component]
fn QueryInput<S>(
    cx: Scope,
    #[allow(unused_variables)] serverfn: std::marker::PhantomData<S>,
) -> impl IntoView
where
    S: ToString + FromStr + Hash + Eq + Clone + leptos::server_fn::ServerFn<leptos::Scope>,
    <S as std::str::FromStr>::Err: Display,
    ServerQueryVal<<S as leptos::server_fn::ServerFn<leptos::Scope>>::Output>:
        Clone + Serializable + 'static,
{
    let (query, set_query) = use_signal::<S>(cx);
    let (input_err, set_input_err) = create_signal(cx, None::<String>);
    view! { cx,
        <label for="flake-url">
            "Load a Nix flake"
            <input
                list="some-flakes"
                id="flake-url"
                type="text"
                class="w-full p-1 font-mono"
                on:change=move |ev| {
                    match FromStr::from_str(&event_target_value(&ev)) {
                        Ok(url) => {
                            set_query(url);
                            set_input_err(None)
                        }
                        Err(e) => set_input_err(Some(e.to_string())),
                    }
                }

                prop:value=move || query().to_string()
            /> <span class="text-red-500">{input_err}</span>
        </label>
    }
}

/// Nix flake dashboard
#[component]
fn NixFlake(cx: Scope) -> impl IntoView {
    let title = "Nix Flake";
    // TODO: make a component
    let (flake_url, _) = use_signal::<GetNixFlake>(cx);
    let res = query::use_server_query::<GetNixFlake>(cx, flake_url);
    view! { cx,
        <Title text=title/>
        <h1 class="text-5xl font-bold">{title}</h1>
        <QueryInput serverfn=(PhantomData::<GetNixFlake>)/>
        // TODO: use local storage, and cache user's inputs
        <datalist id="some-flakes">
            <option value="github:nammayatri/nammayatri"></option>
            <option value="github:srid/haskell-template"></option>
            <option value="github:juspay/nix-browser"></option>
            <option value="github:nixos/nixpkgs"></option>
        </datalist>
        <RefetchQueryButton res=res.clone() k=()/>
        <div class="my-1 text-left">
            // <SuspenseWithErrorHandling>{res.data}</SuspenseWithErrorHandling>
            <Suspense fallback=move || view! { cx, <Spinner/> }>
                <ErrorBoundary fallback=|cx, errors| {
                    view! { cx, <Errors errors=errors.get()/> }
                }>{res.data}</ErrorBoundary>
            </Suspense>
        </div>
    }
}

/// Nix information
#[component]
fn NixInfo(cx: Scope) -> impl IntoView {
    let title = "Nix Info";
    let res = query::use_nix_info_query(cx);
    view! { cx,
        <Title text=title/>
        <h1 class="text-5xl font-bold">{title}</h1>
        <RefetchQueryButton res=res.clone() k=()/>
        <div class="my-1 text-left">
            <SuspenseWithErrorHandling>{res.data}</SuspenseWithErrorHandling>
        </div>
    }
}

/// Nix health checks
#[component]
fn NixHealth(cx: Scope) -> impl IntoView {
    let title = "Nix Health";
    let res = query::use_nix_health_query(cx);
    view! { cx,
        <Title text=title/>
        <h1 class="text-5xl font-bold">{title}</h1>
        <RefetchQueryButton res=res.clone() k=()/>
        <div class="my-1">
            <SuspenseWithErrorHandling>{res.data}</SuspenseWithErrorHandling>
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
