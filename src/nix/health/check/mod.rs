//! Individual Nix checks
pub mod caches;
pub mod max_jobs;

use leptos::*;

use crate::nix::health::{
    report::{Report, WithDetails},
    traits::Check,
};

/// View common to rendering all checks
#[component]
pub fn ViewCheck<C>(cx: Scope, check: C, children: Children) -> impl IntoView
where
    C: Check<Report = Report<WithDetails>> + Clone,
{
    let report = (&check).report();
    view! { cx,
        <div class="bg-white border-2 rounded">
            <h2 class="p-2 text-xl font-bold ">
                {report.without_details()} {" "} {(&check).name()}
            </h2>
            <div class="p-2 ">
                <div class="py-2 my-2 bg-base-50">{children(cx)}</div>
                <div class="flex flex-col justify-start space-y-4">{report.get_red_details()}</div>
            </div>
        </div>
    }
}
