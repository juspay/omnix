//! Frontend UI entry point

use std::collections::BTreeMap;

use leptos::*;
use leptos_extra::{
    query::{self, RefetchQueryButton},
    signal::{use_signal, SignalWithResult},
};
use leptos_meta::*;
use leptos_router::*;
use nix_rs::flake::{
    outputs::{FlakeOutputs, Val},
    schema::FlakeSchema,
    url::FlakeUrl,
    Flake,
};
use nix_rs::{command::Refresh, flake::outputs::Type};
use tracing::instrument;

use crate::widget::*;

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
                {move || data.with_result(move |r| view_flake_outputs(cx, r.clone().output))}
            </SuspenseWithErrorHandling>
        </div>
    }
}

fn view_flake(cx: Scope, flake: Flake) -> View {
    view! { cx,
        <div class="flex flex-col my-4">
            <h3 class="text-lg font-bold">{flake.url.to_string()}</h3>
            <div class="text-sm italic text-gray-600">
                <A href="/flake/raw" exact=true>
                    "View raw output"
                </A>
            </div>
            <div>{view_flake_schema(cx, flake.schema)}</div>
        </div>
    }
    .into_view(cx)
}

fn view_flake_schema(cx: Scope, schema: FlakeSchema) -> View {
    let system = &schema.system.clone();
    fn view_section_heading(cx: Scope, title: &'static str) -> impl IntoView {
        view! { cx,
            <h3 class="p-2 mt-4 mb-2 font-bold bg-gray-300 border-b-2 border-l-2 border-black text-l">
                {title}
            </h3>
        }
    }
    fn view_btree(cx: Scope, title: &'static str, tree: &BTreeMap<String, Val>) -> impl IntoView {
        (!tree.is_empty()).then(|| {
            view! { cx,
                {view_section_heading(cx, title)}
                {view_btree_body(cx, tree)}
            }
        })
    }
    view! { cx,
        <div>
            <h2 class="my-2 ">
                <div class="text-xl font-bold text-primary-600">{system.human_readable()}</div>
                " "
                <span class="font-mono text-xs text-gray-500">"(" {system.to_string()} ")"</span>
            </h2>

            <div class="text-left">
                {view_btree(cx, "Packages", &schema.packages)}
                {view_btree(cx, "Legacy Packages", &schema.legacy_packages)}
                {view_btree(cx, "Dev Shells", &schema.devshells)}
                {view_btree(cx, "Checks", &schema.checks)} {view_btree(cx, "Apps", &schema.apps)}
                {view_section_heading(cx, "Formatter")}
                {schema
                    .formatter
                    .map(|v| {
                        let default = "formatter".to_string();
                        let k = v.name.as_ref().unwrap_or(&default);
                        view_flake_val(cx, k, &v)
                    })}
                {view_section_heading(cx, "Other")}
                {schema.other.map(|v| view_flake_outputs(cx, FlakeOutputs::Attrset(v)))}
            </div>
        </div>
    }
    .into_view(cx)
}

fn view_btree_body(cx: Scope, tree: &BTreeMap<String, Val>) -> View {
    view! { cx,
        <div class="flex flex-wrap justify-start">
            {tree.iter().map(|(k, v)| view_flake_val(cx, k, v)).collect_view(cx)}
        </div>
    }
    .into_view(cx)
}

fn view_flake_val(cx: Scope, k: &String, v: &Val) -> impl IntoView {
    view! { cx,
        <div
            title=format!("{:?}", v.type_)
            class="flex flex-col p-2 my-2 mr-2 space-y-2 bg-white border-4 border-gray-300 rounded hover:border-gray-400"
        >
            <div class="flex flex-row justify-start space-x-2 font-bold text-primary-500">
                <div>{v.type_.to_icon()}</div>
                <div>{k}</div>
            </div>
            {v
                .name
                .as_ref()
                .map(|v| {
                    view! { cx, <div class="font-mono text-xs text-gray-500">{v}</div> }
                })}

            {v
                .description
                .as_ref()
                .map(|v| {
                    view! { cx, <div class="font-light">{v}</div> }
                })}

        </div>
    }
}

/// The [IntoView] instance for [FlakeOutputs] renders it recursively. This view
/// is used to see the raw flake output only; it is not useful for general UX.
///
/// WARNING: This may cause performance problems if the tree is large.
fn view_flake_outputs(cx: Scope, outs: FlakeOutputs) -> View {
    match outs {
        FlakeOutputs::Val(v) => view_val(cx, v),
        FlakeOutputs::Attrset(v) => view! { cx,
            <ul class="list-disc">
                {v
                    .iter()
                    .map(|(k, v)| {
                        view! { cx,
                            <li class="ml-4">
                                <span class="px-2 py-1 font-bold text-primary-500">{k}</span>
                                {view_flake_outputs(cx, v.clone())}
                            </li>
                        }
                    })
                    .collect_view(cx)}
            </ul>
        }
        .into_view(cx),
    }
}

fn view_val(cx: Scope, val: Val) -> View {
    view! { cx,
        <span>
            <b>{val.name}</b>
            " ("
            {view_type(cx, val.type_)}
            ") "
            <em>{val.description}</em>
        </span>
    }
    .into_view(cx)
}

fn view_type(cx: Scope, type_: Type) -> View {
    view! { cx,
        <span>
            {match type_ {
                Type::NixosModule => "nixosModule ❄️",
                Type::Derivation => "derivation 📦",
                Type::App => "app 📱",
                Type::Template => "template 🏗️",
                Type::Unknown => "unknown ❓",
            }}

        </span>
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
