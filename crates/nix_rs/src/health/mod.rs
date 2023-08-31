//! Health checks for the user's Nix install

mod check;
pub mod report;
pub mod traits;

use leptos::*;
use serde::{Deserialize, Serialize};

use self::check::{
    caches::Caches, flake_enabled::FlakeEnabled, max_jobs::MaxJobs, min_nix_version::MinNixVersion,
};
use self::report::{NoDetails, Report, WithDetails};
use self::traits::Check;
use super::info;

/// Nix Health check information for user's install
///
/// Each field represents an individual check which satisfies the [Check] trait.
///
/// NOTE: This struct is isomorphic to [Vec<Box<&dyn Check>>]. We cannot use the
/// latter due to (wasm) serialization limitation with dyn trait objects. An
// [IntoIterator] impl is provide towards this end.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NixHealth {
    max_jobs: MaxJobs,
    caches: Caches,
    flake_enabled: FlakeEnabled,
    min_nix_version: MinNixVersion,
}

impl<'a> IntoIterator for &'a NixHealth {
    type Item = &'a dyn Check<Report = Report<WithDetails>>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    /// Return an iterator to iterate on the fields of [NixHealth]
    fn into_iter(self) -> Self::IntoIter {
        let items: Vec<Self::Item> = vec![
            &self.max_jobs,
            &self.caches,
            &self.flake_enabled,
            &self.min_nix_version,
        ];
        items.into_iter()
    }
}

impl Check for NixHealth {
    type Report = Report<NoDetails>;
    fn check(info: &info::NixInfo) -> Self {
        NixHealth {
            max_jobs: MaxJobs::check(info),
            caches: Caches::check(info),
            flake_enabled: FlakeEnabled::check(info),
            min_nix_version: MinNixVersion::check(info),
        }
    }
    fn name(&self) -> &'static str {
        "Nix Health"
    }
    fn report(&self) -> Report<NoDetails> {
        if self.into_iter().all(|c| c.report() == Report::Green) {
            Report::Green
        } else {
            Report::Red(NoDetails)
        }
    }
}

impl IntoView for NixHealth {
    fn into_view(self, cx: Scope) -> View {
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
                            {report.without_details()} {" "} {check.name()}
                        </summary>
                        <div class="p-4">
                            <div class="p-2 my-2 font-mono text-sm bg-black text-base-100">
                                {check}
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
            <div class="flex flex-col items-stretch justify-start space-y-8 text-left">
                // TODO: Make this use [NixHealth::into_iter]
                <ViewCheck check=self.min_nix_version/>
                <ViewCheck check=self.max_jobs/>
                <ViewCheck check=self.caches/>
                <ViewCheck check=self.flake_enabled/>
            </div>
        }
        .into_view(cx)
    }
}
