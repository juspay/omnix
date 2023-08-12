use leptos::*;
use serde::{Deserialize, Serialize};

/// Health report
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Clone)]
pub enum Report<T> {
    /// Green means everything is fine
    Green,
    /// Red means something is wrong. [T] holds information about what's wrong.
    Red(T),
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Clone)]
pub struct NoDetails;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Clone)]
pub struct WithDetails {
    /// A short message describing the problem
    pub msg: &'static str,
    /// A suggestion for how to fix the problem
    pub suggestion: &'static str,
} // TODO: Should this be Markdown?

impl Report<WithDetails> {
    /// Return the report without the details
    pub fn indicator_only(&self) -> Report<NoDetails> {
        match self {
            Report::Green => Report::Green,
            Report::Red(_) => Report::Red(NoDetails),
        }
    }
    /// Return the problem details if there is one.
    pub fn red_details_only(&self) -> Option<WithDetails> {
        match self {
            Report::Green => None,
            Report::Red(details) => Some(details.clone()),
        }
    }
}

impl IntoView for WithDetails {
    fn into_view(self, cx: Scope) -> View {
        view! { cx,
            {}
            <div class="bg-red-400 rounded bg-border">{self.msg}</div>
            <div class="bg-blue-400 rounded bg-border">"Suggestion: " {self.suggestion}</div>
        }
        .into_view(cx)
    }
}

impl IntoView for Report<WithDetails> {
    fn into_view(self, cx: Scope) -> View {
        view! { cx,
            {self.indicator_only()}
            {self.red_details_only()}
        }
        .into_view(cx)
    }
}

impl IntoView for Report<NoDetails> {
    fn into_view(self, cx: Scope) -> View {
        view! { cx,
            {match self {
                Report::Green => {
                    view! { cx, <div class="text-green-500">{"✓"}</div> }.into_view(cx)
                }
                Report::Red(NoDetails) => {

                    view! { cx, <div class="text-3xl text-red-500">{"✗"}</div> }
                        .into_view(cx)
                }
            }}
        }
        .into_view(cx)
    }
}
