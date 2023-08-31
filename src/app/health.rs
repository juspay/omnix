//! Nix health check UI

use leptos::*;
use leptos_meta::*;
use nix_rs::{
    health::{
        check::{
            caches::Caches, flake_enabled::FlakeEnabled, max_jobs::MaxJobs,
            min_nix_version::MinNixVersion,
        },
        report::{NoDetails, Report, WithDetails},
        traits::Check,
        NixHealth,
    },
    version::NixVersion,
};
use tracing::instrument;

use crate::{app::info::ConfigValListView, widget::*};
use leptos_extra::{
    query::{self, RefetchQueryButton},
    signal::SignalWithResult,
};

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
    #[component]
    fn ViewCheck<C>(cx: Scope, check: C, children: Children) -> impl IntoView
    where
        C: Check<Report = Report<WithDetails>>,
    {
        let report = check.report();
        view! { cx,
            <div class="contents">
                <details
                    open=report != Report::Green
                    class="my-2 bg-white border-2 rounded-lg cursor-pointer hover:bg-primary-100 border-base-300"
                >
                    <summary class="p-4 text-xl font-bold">
                        <ReportSummaryView report=&report.without_details()/>
                        {" "}
                        {check.name()}
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
    view! { cx,
        <div class="flex flex-col items-stretch justify-start space-y-8 text-left">
            <ViewCheck check=health.min_nix_version.clone()>
                <MinNixVersionView v=health.min_nix_version/>
            </ViewCheck>
            <ViewCheck check=health.max_jobs.clone()>
                <MaxJobsView v=health.max_jobs/>
            </ViewCheck>
            <ViewCheck check=health.caches.clone()>
                <CachesView v=health.caches/>
            </ViewCheck>
            <ViewCheck check=health.flake_enabled.clone()>
                <FlakeEnabledView v=health.flake_enabled/>
            </ViewCheck>
        </div>
    }
}

#[component]
fn CachesView(cx: Scope, v: Caches) -> impl IntoView {
    view! { cx, <div>"The following caches are in use:" <ConfigValListView cfg=v.0/></div> }
}

#[component]
fn FlakeEnabledView(cx: Scope, v: FlakeEnabled) -> impl IntoView {
    view! { cx, <span>"experimental-features: " <ConfigValListView cfg=v.0/></span> }
}

#[component]
fn MaxJobsView(cx: Scope, v: MaxJobs) -> impl IntoView {
    view! { cx, <span>"Nix builds are using " {v.0.value} " cores"</span> }
}

#[component]
fn MinNixVersionView(cx: Scope, v: MinNixVersion) -> impl IntoView {
    view! { cx, <span>"Nix version: " <NixVersionView ver=v.0/></span> }
}

#[component]
fn NixVersionView(cx: Scope, ver: NixVersion) -> impl IntoView {
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
pub async fn get_nix_health(_unit: ()) -> Result<nix_rs::health::NixHealth, ServerFnError> {
    use nix_rs::{
        health::{traits::Check, NixHealth},
        info,
    };
    let info = info::NixInfo::from_nix(&nix_rs::command::NixCmd::default()).await?;
    let health = NixHealth::check(&info);
    Ok(health)
}
