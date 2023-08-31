//! Frontend UI entry point

use leptos::*;
use leptos_extra::signal::SignalWithResult;
use leptos_meta::*;
use nix_rs::{
    config::{ConfigVal, NixConfig},
    info::NixInfo,
    version::NixVersion,
};

use crate::widget::*;
use leptos_extra::query::{self, RefetchQueryButton};

/// Nix information
#[component]
pub fn NixInfoRoute(cx: Scope) -> impl IntoView {
    let title = "Nix Info";
    let result = query::use_server_query(cx, || (), get_nix_info);
    let data = result.data;
    view! { cx,
        <Title text=title/>
        <h1 class="text-5xl font-bold">{title}</h1>
        <RefetchQueryButton result query=|| ()/>
        <div class="my-1 text-left">
            <SuspenseWithErrorHandling>
                {move || {
                    data.with_result(move |info| {
                        view! { cx, <NixInfoView info/> }
                    })
                }}

            </SuspenseWithErrorHandling>
        </div>
    }
}

/// Determine [NixInfo] on the user's system
#[server(GetNixInfo, "/api")]
pub async fn get_nix_info(_unit: ()) -> Result<NixInfo, ServerFnError> {
    let v = NixInfo::from_nix(&nix_rs::command::NixCmd::default()).await?;
    Ok(v)
}

#[component]
fn NixInfoView<'a>(cx: Scope, info: &'a NixInfo) -> impl IntoView {
    view! { cx,
        <div class="flex flex-col p-4 space-y-8 bg-white border-2 rounded border-base-400">
            <div>
                <b>
                    Nix Version
                </b>
                <div class="p-1 my-1 rounded bg-primary-50">
                    <NixVersionView version=&info.nix_version/>
                </div>
            </div>
            <div>
                <b>
                    Nix Config
                </b>
                <NixConfigView config=info.nix_config.clone()/>
            </div>
        </div>
    }
}

#[component]
fn NixVersionView<'a>(cx: Scope, version: &'a NixVersion) -> impl IntoView {
    view! { cx,
        <a href=nix_rs::refs::RELEASE_HISTORY class="font-mono hover:underline" target="_blank">
            {format!("{}", version)}
        </a>
    }
}

#[component]
fn NixConfigView(cx: Scope, config: NixConfig) -> impl IntoView {
    #[component]
    fn ConfigRow<T>(cx: Scope, key: &'static str, value: ConfigVal<T>) -> impl IntoView
    where
        ConfigVal<T>: IntoView,
    {
        view! { cx,
            // TODO: Use a nice Tailwind tooltip here, instead of "title"
            // attribute.
            <tr title=&value.description>
                <td class="px-4 py-2 font-semibold text-base-700">{key}</td>
                <td class="px-4 py-2 text-left">
                    <code>{value}</code>
                </td>
            </tr>
        }
    }
    view! { cx,
        <div class="py-1 my-1 rounded bg-primary-50">
            <table class="text-right">
                <tbody>
                    <ConfigRow key="Local System" value=config.system/>
                    <ConfigRow key="Max Jobs" value=config.max_jobs/>
                    <ConfigRow key="Cores per build" value=config.cores/>
                    <ConfigRow key="Nix Caches" value=config.substituters/>
                </tbody>
            </table>
        </div>
    }
    .into_view(cx)
}
