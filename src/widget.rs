//! Various Leptos widgets

use cfg_if::cfg_if;
#[cfg(feature = "ssr")]
use http::status::StatusCode;
use leptos::*;
#[cfg(feature = "ssr")]
use leptos_axum::ResponseOptions;
use leptos_router::*;

// A loading spinner
#[component]
pub fn Spinner(cx: Scope) -> impl IntoView {
    view! { cx,
        <div
            class="animate-spin inline-block w-6 h-6 border-[3px] border-current border-t-transparent text-blue-600 rounded-full"
            role="status"
            aria-label="loading"
        >
            <span class="sr-only">"Loading..."</span>
        </div>
    }
}

/// A `<a>` link
#[component]
pub fn Link(cx: Scope, link: &'static str, text: &'static str) -> impl IntoView {
    view! { cx,
        <A href=link class="text-primary-100 hover:no-underline">
            {text}
        </A>
    }
}

/// A `<a>` link that links to an external site
#[component]
pub fn LinkExternal(cx: Scope, link: &'static str, text: &'static str) -> impl IntoView {
    view! { cx,
        <a
            href=link
            class="underline text-primary-500 hover:no-underline"
            rel="external"
            target="_blank"
        >
            {text}
        </a>
    }
}

/// 404 page
#[component]
pub fn NotFound(cx: Scope) -> impl IntoView {
    cfg_if! { if #[cfg(feature="ssr")] {
        if let Some(response) = use_context::<ResponseOptions>(cx) {
            response.set_status(StatusCode::NOT_FOUND);
        }
    }}
    view! { cx,
        // The HTML for 404 not found
        <div class="grid w-full min-h-screen bg-center bg-cover bg-base-100 place-items-center">
            <div class="z-0 flex items-center justify-center col-start-1 row-start-1 text-center">
                <div class="flex flex-col space-y-3">
                    <h1 class="text-5xl font-bold">"404"</h1>
                    <p class="py-6">
                        <h2 class="text-3xl font-bold text-gray-500">"Page not found"</h2>
                        <p class="my-1">"The page you are looking for does not exist."</p>
                    </p>
                    <Link link="/" text="Go to home page"/>
                </div>
            </div>
        </div>
    }
}

/// Display errors to the user
#[component]
pub fn Errors(cx: Scope, errors: Errors) -> impl IntoView {
    view! { cx,
        <div class="flex flex-row justify-center overflow-auto text-xl text-white bg-error-500">
            <div class="font-mono whitespace-pre-wrap">
                <ul>
                    {errors
                        .into_iter()
                        .map(|(_, e)| view! { cx, <li>{e.to_string()}</li> })
                        .collect_view(cx)}
                </ul>
            </div>
        </div>
    }
}

/// Like [Suspense] but also handles errors using [ErrorBoundary]
#[component(transparent)]
pub fn SuspenseWithErrorHandling(cx: Scope, children: ChildrenFn) -> impl IntoView {
    let children = store_value(cx, children);
    view! { cx,
        <Suspense fallback=move || view! { cx, <Spinner/> }>
            <ErrorBoundary fallback=|cx, errors| {
                view! { cx, <Errors errors=errors.get()/> }
            }>{children.with_value(|c| c(cx))}</ErrorBoundary>
        </Suspense>
    }
}
