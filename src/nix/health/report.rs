use leptos::*;
use serde::{Deserialize, Serialize};

/// Health report
///
/// If you just want the binary indicator, use `Report<NoDetails>` (see
/// [NoDetails]). If you want the report with details regarding the problem, use
/// `Report<WithDetails>` (see [WithDetails]).
///
/// Reports can be rendered into views using the [IntoView] trait.
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Clone)]
pub enum Report<T> {
    /// Green means everything is fine
    Green,
    /// Red means something is wrong. `T` holds information about what's wrong.
    Red(T),
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Clone)]
pub struct NoDetails;

/// Details regarding a failed report
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Clone)]
pub struct WithDetails {
    /// A short message describing the problem
    pub msg: String,
    /// A suggestion for how to fix the problem
    pub suggestion: String,
} // TODO: Should this be Markdown?

impl Report<WithDetails> {
    /// Return the report without the details
    pub fn without_details(&self) -> Report<NoDetails> {
        match self {
            Report::Green => Report::Green,
            Report::Red(_) => Report::Red(NoDetails),
        }
    }
    /// Return the problem details if there is one.
    pub fn get_red_details(&self) -> Option<WithDetails> {
        match self {
            Report::Green => None,
            Report::Red(details) => Some(details.clone()),
        }
    }
}

impl IntoView for WithDetails {
    fn into_view(self, cx: Scope) -> View {
        view! { cx,
            <h3 class="my-2 font-bold text-l">
                Problem:
            </h3>
            <div class="p-2 bg-red-400 rounded bg-border">{self.msg}</div>
            <h3 class="my-2 font-bold text-l">
                Suggestion:
            </h3>
            <div class="p-2 bg-blue-400 rounded bg-border">{self.suggestion}</div>
        }
        .into_view(cx)
    }
}

impl IntoView for Report<NoDetails> {
    fn into_view(self, cx: Scope) -> View {
        view! { cx,
            {match self {
                Report::Green => {
                    view! { cx, <span class="text-green-500">{"✓"}</span> }.into_view(cx)
                }
                Report::Red(NoDetails) => {

                    view! { cx, <span class="text-red-500">{"✗"}</span> }
                        .into_view(cx)
                }
            }}
        }
        .into_view(cx)
    }
}
