use leptos::*;

use super::{info, report::Report};

pub trait Check: IntoView {
    fn check(info: &info::NixInfo) -> Self
    where
        Self: Sized;

    fn name(&self) -> &'static str;

    fn report(&self) -> Report;
}

#[component]
pub fn ViewCheck<C>(cx: Scope, check: C, children: Children) -> impl IntoView
where
    C: Check + Clone,
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
