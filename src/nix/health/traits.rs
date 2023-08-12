use leptos::*;

use super::{
    info,
    report::{Report, WithDetails},
};

/// Types that implement health check with reports
pub trait Check: IntoView {
    /// The type of the report produced by this health check
    type Report = Report<WithDetails>;
    /// Run and create the health check
    fn check(info: &info::NixInfo) -> Self
    where
        Self: Sized;
    /// User-facing name for this health check
    fn name(&self) -> &'static str;
    /// Return the health report
    fn report(&self) -> Self::Report;
}

#[component]
pub fn ViewCheck<C>(cx: Scope, check: C, children: Children) -> impl IntoView
where
    C: Check<Report = Report<WithDetails>> + Clone,
{
    view! { cx,
        <div class="bg-white border-2 rounded">
            <h2 class="p-2 text-xl font-bold ">{(&check).name()}</h2>
            <div class="p-2">
                <div class="py-2 bg-base-50">{children(cx)}</div>
                <div class="flex flex-col justify-start space-y-8">{(&check).report()}</div>
            </div>
        </div>
    }
}
