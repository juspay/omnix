//! Frontend UI entry point

use leptos::*;
use leptos_extra::signal::{use_signal, SignalWithResult};
use leptos_meta::*;
use leptos_router::*;
use nix_rs::flake::Flake;
use tracing::instrument;

use crate::widget::*;
use leptos_extra::query::{self, RefetchQueryButton};
use nix_rs::command::Refresh;
use nix_rs::flake::url::FlakeUrl;

/// Nix flake dashboard
#[component]
pub fn NixFlake(cx: Scope) -> impl IntoView {
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
pub fn NixFlakeHome(cx: Scope) -> impl IntoView {
    let url = use_signal::<FlakeUrl>(cx);
    let refresh = use_signal::<Refresh>(cx);
    let query = move || (url(), refresh());
    let result = query::use_server_query(cx, query, get_flake);
    let data = result.data;
    view! { cx,
        <div class="p-2 my-1">
            <SuspenseWithErrorHandling>
                {move || data.with_result(move |r| view_flake(cx, r.clone()))}
            </SuspenseWithErrorHandling>
        </div>
    }
}

#[component]
pub fn NixFlakeRaw(cx: Scope) -> impl IntoView {
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
                {move || data.with_result(move |r| r.clone().output.into_view(cx))}
            </SuspenseWithErrorHandling>
        </div>
    }
}

fn view_flake(cx: Scope, flake: Flake) -> View {
    view! { cx,
        <div class="flex flex-col my-4">
            <h3 class="text-lg font-bold">{flake.url}</h3>
            <div class="text-sm italic text-gray-600">
                <A href="/flake/raw" exact=true>
                    "View raw output"
                </A>
            </div>
            <div>{flake.schema}</div>
        </div>
    }
    .into_view(cx)
}

/// Get [Flake] info for the given flake url
#[instrument(name = "flake")]
#[server(GetFlake, "/api")]
pub async fn get_flake(args: (FlakeUrl, Refresh)) -> Result<Flake, ServerFnError> {
    use nix_rs::config::run_nix_show_config;
    let (url, refresh) = args;
    // TODO: Can we cache this?
    let nix_config = run_nix_show_config().await?;
    let system = nix_config.system.value;
    let output = nix_rs::flake::show::run_nix_flake_show(&url, refresh).await?;
    Ok(Flake {
        url,
        output: output.clone(),
        schema: nix_rs::flake::schema::FlakeSchema::from(&output, &system),
    })
}
