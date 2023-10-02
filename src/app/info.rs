//! Nix info UI

use std::fmt::Display;

use leptos::*;
use leptos_extra::query::{self, RefetchQueryButton};
use leptos_extra::signal::SignalWithResult;
use leptos_meta::*;
use nix_rs::{
    config::{ConfigVal, NixConfig},
    info::NixInfo,
    version::NixVersion,
};

use crate::widget::*;

/// Nix information
#[component]
pub fn NixInfoRoute() -> impl IntoView {
    let title = "Nix Info";
    let result = query::use_server_query(|| (), get_nix_info);
    let data = result.data;
    view! {
        <Title text=title/>
        <h1 class="text-5xl font-bold">{title}</h1>
        <RefetchQueryButton result query=|| ()/>
        <div class="my-1 text-left">
            <SuspenseWithErrorHandling>
                {move || {
                    data.with_result(move |info| {
                        view! { <NixInfoView info/> }
                    })
                }}

            </SuspenseWithErrorHandling>
        </div>
    }
}

#[component]
fn NixInfoView<'a>(info: &'a NixInfo) -> impl IntoView {
    view! {
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
fn NixVersionView<'a>(version: &'a NixVersion) -> impl IntoView {
    view! {
        <a href=nix_rs::refs::RELEASE_HISTORY class="font-mono hover:underline" target="_blank">
            {format!("{}", version)}
        </a>
    }
}

#[component]
fn NixConfigView(config: NixConfig) -> impl IntoView {
    #[component]
    fn ConfigRow(key: &'static str, title: String, children: Children) -> impl IntoView {
        view! {
            <tr title=title>
                <td class="px-4 py-2 font-semibold text-base-700">{key}</td>
                <td class="px-4 py-2 text-left">
                    <code>{children()}</code>
                </td>
            </tr>
        }
    }
    view! {
        <div class="py-1 my-1 rounded bg-primary-50">
            <table class="text-right">
                // FIXME: so many clones
                <tbody>
                    <ConfigRow key="Local System" title=config.system.description>
                        {config.system.value.to_string()}
                    </ConfigRow>
                    <ConfigRow key="Max Jobs" title=config.max_jobs.description.clone()>
                        {config.max_jobs.value}
                    </ConfigRow>
                    <ConfigRow key="Cores per build" title=config.cores.description>
                        {config.cores.value}
                    </ConfigRow>
                    <ConfigRow key="Nix Caches" title=config.substituters.clone().description>
                        <ConfigValListView cfg=config.substituters.clone()/>
                    </ConfigRow>
                </tbody>
            </table>
        </div>
    }
    .into_view()
}

#[component]
pub fn ConfigValListView<T>(cfg: ConfigVal<Vec<T>>) -> impl IntoView
where
    T: Display,
{
    view! {
        <div class="flex flex-col space-y-4">
            {cfg
                .value
                .into_iter()
                .map(|item| view! { <li class="list-disc">{item.to_string()}</li> })
                .collect_view()}
        </div>
    }
    .into_view()
}

/// Determine [NixInfo] on the user's system
#[server(GetNixInfo, "/api")]
pub async fn get_nix_info(_unit: ()) -> Result<NixInfo, ServerFnError> {
    let v = NixInfo::from_nix(&nix_rs::command::NixCmd::default()).await?;
    Ok(v)
}
