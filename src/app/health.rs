//! Nix health check UI

use leptos::*;
use leptos_extra::{
    query::{self, RefetchQueryButton},
    signal::SignalWithResult,
};
use leptos_meta::*;
use nix_health::{
    check::{
        caches::Caches, flake_enabled::FlakeEnabled, max_jobs::MaxJobs,
        min_nix_version::MinNixVersion, trusted_users::TrustedUsers,
    },
    report::{NoDetails, Report, WithDetails},
    traits::Check,
    NixHealth,
};
use nix_rs::version::NixVersion;
use tracing::instrument;

use crate::{app::info::ConfigValListView, widget::*};

/// Nix health checks
#[component]
pub fn NixHealthRoute(cx: Scope) -> impl IntoView {
    let title = "Nix Health";
    let result = query::use_server_query(cx, || (), get_nix_health);
    let data = result.data;
    view! { cx,
        <Title text=title/>
        <h1 class="text-5xl font-bold">{title}</h1>
        <RefetchQueryButton result query=|| ()/>
        <div class="my-1">
            <SuspenseWithErrorHandling>
                {move || {
                    data.with_result(move |health| {
                        view! { cx, <NixHealthView health=health.clone()/> }
                    })
                }}

            </SuspenseWithErrorHandling>
        </div>
    }
}

#[component]
fn NixHealthView(cx: Scope, health: NixHealth) -> impl IntoView {
    view! { cx,
        <div class="flex flex-col items-stretch justify-start space-y-8 text-left">
            // NOTE: Aim to keep this order in alignment with that of the IntoIterator impl
            <ViewCheck name=health.min_nix_version.name() report=health.min_nix_version.report()>
                <MinNixVersionView v=&health.min_nix_version/>
            </ViewCheck>
            <ViewCheck name=health.flake_enabled.name() report=health.flake_enabled.report()>
                <FlakeEnabledView v=&health.flake_enabled/>
            </ViewCheck>
            <ViewCheck name=health.max_jobs.name() report=health.max_jobs.report()>
                <MaxJobsView v=&health.max_jobs/>
            </ViewCheck>
            <ViewCheck name=health.caches.name() report=health.caches.report()>
                <CachesView v=&health.caches/>
            </ViewCheck>
            <ViewCheck name=health.trusted_users.name() report=health.trusted_users.report()>
                <TrustedUsersView v=&health.trusted_users/>
            </ViewCheck>
        </div>
    }
}

#[component]
fn ViewCheck(
    cx: Scope,
    name: &'static str,
    report: Report<WithDetails>,
    children: Children,
) -> impl IntoView {
    view! { cx,
        <div class="contents">
            <details
                open=report != Report::Green
                class="my-2 bg-white border-2 rounded-lg cursor-pointer hover:bg-primary-100 border-base-300"
            >
                <summary class="p-4 text-xl font-bold">
                    <ReportSummaryView report=&report.without_details()/>
                    {" "}
                    {name}
                </summary>
                <div class="p-4">
                    <div class="p-2 my-2 font-mono text-sm bg-black text-base-100">
                        {children(cx)}
                    </div>
                    <div class="flex flex-col justify-start space-y-4">
                        {report
                            .get_red_details()
                            .map(move |details| {
                                view! { cx, <WithDetailsView details/> }
                            })}

                    </div>
                </div>
            </details>
        </div>
    }
}

#[component]
fn CachesView<'a>(cx: Scope, v: &'a Caches) -> impl IntoView {
    view! { cx, <div>"The following caches are in use:" <ConfigValListView cfg=v.0.clone()/></div> }
}

#[component]
fn FlakeEnabledView<'a>(cx: Scope, v: &'a FlakeEnabled) -> impl IntoView {
    view! { cx, <span>"experimental-features: " <ConfigValListView cfg=v.0.clone()/></span> }
}

#[component]
fn TrustedUsersView<'a>(cx: Scope, v: &'a TrustedUsers) -> impl IntoView {
    view! { cx, <span>"trusted_users: " <ConfigValListView cfg=v.trusted_users.clone()/></span> }
}

#[component]
fn MaxJobsView<'a>(cx: Scope, v: &'a MaxJobs) -> impl IntoView {
    view! { cx, <span>"Nix builds are using " {v.0.value} " cores"</span> }
}

#[component]
fn MinNixVersionView<'a>(cx: Scope, v: &'a MinNixVersion) -> impl IntoView {
    view! { cx, <span>"Nix version: " <NixVersionView ver=&v.0/></span> }
}

#[component]
fn NixVersionView<'a>(cx: Scope, ver: &'a NixVersion) -> impl IntoView {
    view! { cx,
        <a href=nix_rs::refs::RELEASE_HISTORY class="font-mono hover:underline" target="_blank">
            {format!("{}", ver)}
        </a>
    }
}

#[component]
pub fn ReportSummaryView<'a>(cx: Scope, report: &'a Report<NoDetails>) -> impl IntoView {
    view! { cx,
        {match report {
            Report::Green => view! { cx, <span class="text-green-500">{"✓"}</span> }.into_view(cx),
            Report::Red(NoDetails) => {

                view! { cx, <span class="text-red-500">{"✗"}</span> }
                    .into_view(cx)
            }
        }}
    }
}

#[component]
fn WithDetailsView(cx: Scope, details: WithDetails) -> impl IntoView {
    view! { cx,
        <h3 class="my-2 font-bold text-l"></h3>
        <div class="p-2 bg-red-400 rounded bg-border">{details.msg}</div>
        <h3 class="my-2 font-bold text-l"></h3>
        <div class="p-2 bg-blue-400 rounded bg-border">{details.suggestion}</div>
    }
}

/// Get [NixHealth] information
#[instrument(name = "nix-health")]
#[server(GetNixHealth, "/api")]
pub async fn get_nix_health(_unit: ()) -> Result<nix_health::NixHealth, ServerFnError> {
    use nix_health::{traits::Check, NixHealth};
    use nix_rs::{env, info};
    let nix_info = info::NixInfo::from_nix(&nix_rs::command::NixCmd::default()).await?;
    let nix_env = env::NixEnv::get_info().await?;
    let health = NixHealth::check(&nix_info, &nix_env);
    Ok(health)
}
