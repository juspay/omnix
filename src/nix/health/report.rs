use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Clone)]
pub enum Report {
    Green,
    Red {
        msg: &'static str,
        suggestion: &'static str,
    }, // TODO: Should this be Markdown?
}

impl IntoView for Report {
    fn into_view(self, cx: Scope) -> View {
        view! { cx,
            {match self {
                Report::Green => {
                    view! { cx, <div class="text-green-500">{"✓"}</div> }.into_view(cx)
                }
                Report::Red { msg, suggestion } => {

                    view! { cx,
                        <div class="text-3xl text-red-500">{"✗"}</div>
                        <div class="bg-red-400 rounded bg-border">{msg}</div>
                        <div class="bg-blue-400 rounded bg-border">"Suggestion: " {suggestion}</div>
                    }
                        .into_view(cx)
                }
            }}
        }
        .into_view(cx)
    }
}
