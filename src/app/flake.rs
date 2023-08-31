//! UI for /flake segment of the app

use std::collections::BTreeMap;

use leptos::*;
use leptos_extra::{
    query::{self, RefetchQueryButton},
    signal::{use_signal, SignalWithResult},
};
use leptos_meta::*;
use leptos_router::*;
use nix_rs::{
    command::Refresh,
    flake::{
        outputs::{FlakeOutputs, Type, Val},
        schema::FlakeSchema,
        url::FlakeUrl,
        Flake,
    },
};

use crate::widget::*;

/// Nix flake dashboard
#[component]
pub fn NixFlakeRoute(cx: Scope) -> impl IntoView {
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
pub fn NixFlakeHomeRoute(cx: Scope) -> impl IntoView {
    let url = use_signal::<FlakeUrl>(cx);
    let refresh = use_signal::<Refresh>(cx);
    let query = move || (url(), refresh());
    let result = query::use_server_query(cx, query, get_flake);
    let data = result.data;
    view! { cx,
        <div class="p-2 my-1">
            <SuspenseWithErrorHandling>
                {move || {
                    data.with_result(move |flake| {
                        view! { cx, <FlakeView flake/> }
                    })
                }}

            </SuspenseWithErrorHandling>
        </div>
    }
}

#[component]
pub fn NixFlakeRawRoute(cx: Scope) -> impl IntoView {
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
                {move || {
                    data.with_result(move |r| {
                        view! { cx, <FlakeOutputsRawView outs=&r.output/> }
                    })
                }}

            </SuspenseWithErrorHandling>
        </div>
    }
}

#[component]
fn FlakeView<'a>(cx: Scope, flake: &'a Flake) -> impl IntoView {
    view! { cx,
        <div class="flex flex-col my-4">
            <h3 class="text-lg font-bold">{flake.url.to_string()}</h3>
            <div class="text-sm italic text-gray-600">
                <A href="/flake/raw" exact=true>
                    "View raw output"
                </A>
            </div>
            <div>
                <FlakeSchemaView schema=&flake.schema/>
            </div>
        </div>
    }
}

#[component]
fn SectionHeading(cx: Scope, title: &'static str) -> impl IntoView {
    view! { cx,
        <h3 class="p-2 mt-4 mb-2 font-bold bg-gray-300 border-b-2 border-l-2 border-black text-l">
            {title}
        </h3>
    }
}

#[component]
fn FlakeSchemaView<'a>(cx: Scope, schema: &'a FlakeSchema) -> impl IntoView {
    let system = &schema.system.clone();
    view! { cx,
        <div>
            <h2 class="my-2 ">
                <div class="text-xl font-bold text-primary-600">{system.human_readable()}</div>
                " "
                <span class="font-mono text-xs text-gray-500">"(" {system.to_string()} ")"</span>
            </h2>

            <div class="text-left">
                <BTreeMapView title="Packages" tree=&schema.packages/>
                <BTreeMapView title="Legacy Packages" tree=&schema.legacy_packages/>
                <BTreeMapView title="Dev Shells" tree=&schema.devshells/>
                <BTreeMapView title="Checks" tree=&schema.checks/>
                <BTreeMapView title="Apps" tree=&schema.apps/>
                <SectionHeading title="Formatter"/>
                {schema
                    .formatter
                    .as_ref()
                    .map(|v| {
                        let default = "formatter".to_string();
                        let k = v.name.as_ref().unwrap_or(&default);
                        view! { cx, <FlakeValView k v/> }
                    })}

                <SectionHeading title="Other"/>
                {schema
                    .other
                    .as_ref()
                    .map(|v| {
                        // TODO: Use a non-recursive rendering component?
                        view! { cx, <FlakeOutputsRawView outs=&FlakeOutputs::Attrset(v.clone())/> }
                    })}

            </div>
        </div>
    }
}

#[component]
fn BTreeMapView<'a>(
    cx: Scope,
    title: &'static str,
    tree: &'a BTreeMap<String, Val>,
) -> impl IntoView {
    (!tree.is_empty()).then(move || {
        view! { cx,
            <SectionHeading title/>
            <BTreeMapBodyView tree/>
        }
    })
}

#[component]
fn BTreeMapBodyView<'a>(cx: Scope, tree: &'a BTreeMap<String, Val>) -> impl IntoView {
    view! { cx,
        <div class="flex flex-wrap justify-start">
            {tree.iter().map(|(k, v)| view! { cx, <FlakeValView k v/> }).collect_view(cx)}
        </div>
    }
}

#[component]
fn FlakeValView<'a>(cx: Scope, k: &'a String, v: &'a Val) -> impl IntoView {
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

/// This component renders recursively. This view is used to see the raw flake
/// output only; it is not useful for general UX.
///
/// WARNING: This may cause performance problems if the tree is large.
#[component]
fn FlakeOutputsRawView<'a>(cx: Scope, outs: &'a FlakeOutputs) -> impl IntoView {
    fn view_val<'b>(cx: Scope, val: &'b Val) -> View {
        view! { cx,
            <span>
                <b>{val.name.clone()}</b>
                " ("
                <TypeView type_=&val.type_/>
                ") "
                <em>{val.description.clone()}</em>
            </span>
        }
        .into_view(cx)
    }

    #[component]
    fn TypeView<'b>(cx: Scope, type_: &'b Type) -> impl IntoView {
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
    }
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
                                <FlakeOutputsRawView outs=v/>
                            </li>
                        }
                    })
                    .collect_view(cx)}
            </ul>
        }
        .into_view(cx),
    }
}

/// Get [Flake] info for the given flake url
#[server(GetFlake, "/api")]
pub async fn get_flake(args: (FlakeUrl, Refresh)) -> Result<Flake, ServerFnError> {
    use nix_rs::command::NixCmd;
    let (url, refresh) = args;
    let nix_cmd = &NixCmd {
        refresh,
        ..NixCmd::default()
    };
    let v = Flake::from_nix(nix_cmd, url).await?;
    Ok(v)
}
