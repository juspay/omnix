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
        div { class: "flex-col items-center justify-center space-y-2 mb-4",
            button {
                class: "py-1 px-2 shadow-lg border-1 {button_cls} rounded-md",
                disabled: *busy,
                onclick: handler,
                "Refresh "
                if *busy {
                    render! { "⏳" }
                } else {
                    render! { "🔄" }
                }
            }
            if *busy {
                render! { Loader {} }
            }
        }
    }
}

#[component]
pub fn Loader(cx: Scope) -> Element {
    render! {
        div { class: "flex justify-center items-center",
            div { class: "animate-spin rounded-full h-16 w-16 border-t-2 border-b-2 border-purple-500" }
        }
    }
}
