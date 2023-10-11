//! Various widgets

use dioxus::prelude::*;

/// A refresh button with a busy indicator
///
/// You want to use [crate::state::datum] for this.
#[component]
pub fn RefreshButton<F>(cx: Scope, busy: bool, handler: F) -> Element
where
    F: Fn(Event<MouseData>),
{
    let button_cls = if *busy {
        "bg-gray-400 text-white"
    } else {
        "bg-blue-700 text-white hover:bg-blue-800"
    };
    render! {
        button {
            class: "p-1 shadow-lg border-1 {button_cls} rounded-md",
            disabled: *busy,
            onclick: handler,
            "Refresh "
            if *busy {
                render! { "⏳" }
            }
        }
    }
}

#[component]
pub fn Loader(cx: Scope) -> Element {
    render! {
        div { class: "flex justify-center items-center",
            div { class: "animate-spin rounded-full h-32 w-32 border-t-2 border-b-2 border-purple-500" }
        }
    }
}
