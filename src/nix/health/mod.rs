//! Health checks for the user's Nix install

mod check;
pub mod report;
pub mod traits;

use leptos::*;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use self::check::{caches::Caches, flake_enabled::FlakeEnabled, max_jobs::MaxJobs};
use self::report::{NoDetails, Report, WithDetails};
use self::traits::Check;
use super::info;

/// Get [NixHealth] information
#[instrument(name = "nix-health")]
#[server(GetNixHealth, "/api")]
pub async fn get_nix_health() -> Result<NixHealth, ServerFnError> {
    let info = info::get_nix_info().await?;
    Ok(NixHealth::check(&info))
}

/// Nix Health check information for user's install
///
/// Each field represents an individual check which satisfies the [Check] trait.
///
/// NOTE: This struct is isomorphic to [Vec<Box<&dyn Check>>]. We cannot use the
/// latter due to (wasm) serialization limitation with dyn trait objects.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NixHealth {
    max_jobs: MaxJobs,
    caches: Caches,
    flake_enabled: FlakeEnabled,
}

impl NixHealth {
    // Return all the fields of the [NixHealth] struct
    pub fn all_checks(&self) -> Vec<&dyn Check<Report = Report<WithDetails>>> {
        vec![&self.max_jobs, &self.caches, &self.flake_enabled]
    }
}

impl Check for NixHealth {
    type Report = Report<NoDetails>;
    fn check(info: &info::NixInfo) -> Self {
        NixHealth {
            max_jobs: MaxJobs::check(info),
            caches: Caches::check(info),
            flake_enabled: FlakeEnabled::check(info),
        }
    }
    fn name(&self) -> &'static str {
        "Nix Health"
    }
    fn report(&self) -> Report<NoDetails> {
        if self
            .all_checks()
            .iter()
            .all(|c| c.report() == Report::Green)
        {
            Report::Green
        } else {
            Report::Red(NoDetails)
        }
    }
}

impl IntoView for NixHealth {
    fn into_view(self, cx: Scope) -> View {
        #[component]
        fn ViewCheck<C>(cx: Scope, check: C, children: Children) -> impl IntoView
        where
            C: Check<Report = Report<WithDetails>>,
        {
            let report = (&check).report();
            view! { cx,
                <div class="contents">
                    <details
                        open=report != Report::Green
                        class="bg-white border-2 my-2 rounded-lg cursor-pointer hover:bg-primary-100 border-2 border-base-300"
                    >
                        <summary class="p-4 text-xl font-bold">
                            {report.without_details()} {" "} {(&check).name()}
                        </summary>
                        <div class="p-4">
                            <div class="p-2 my-2 bg-black text-base-100 font-mono text-sm">
                                {children(cx)}
                            </div>
                            <div class="flex flex-col justify-start space-y-4">
                                {report.get_red_details()}
                            </div>
                        </div>
                    </details>
                </div>
            }
        }
        view! { cx,
            <div class="flex flex-col items-stretch justify-start text-left space-y-8">
                // TODO: Make this use [NixHealth::all_checks]
                <ViewCheck check=self.max_jobs.clone()>{self.max_jobs}</ViewCheck>
                <ViewCheck check=self.caches.clone()>{self.caches}</ViewCheck>
                <ViewCheck check=self.flake_enabled.clone()>{self.flake_enabled}</ViewCheck>
            </div>
        }
        .into_view(cx)
    }
}
