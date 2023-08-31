//! Nix health check UI

use leptos::*;
use leptos_meta::*;
use nix_rs::health::{
    report::{NoDetails, Report, WithDetails},
    traits::Check,
    NixHealth,
};
use tracing::instrument;

use crate::widget::*;
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
    fn ViewCheck<C>(cx: Scope, check: C) -> impl IntoView
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
                        <div class="p-2 my-2 font-mono text-sm bg-black text-base-100">{check}</div>
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
            // TODO: Make this use [NixHealth::into_iter]
            <ViewCheck check=health.min_nix_version/>
            <ViewCheck check=health.max_jobs/>
            <ViewCheck check=health.caches/>
            <ViewCheck check=health.flake_enabled/>
        </div>
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
        <h3 class="my-2 font-bold text-l">
                Prob
        </h3>
        <div class="p-2 bg-red-400 rounded bg-border">{details.msg}</div>
        <h3 class="my-2 font-bold text-l">
                Suggest
        </h3>
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
