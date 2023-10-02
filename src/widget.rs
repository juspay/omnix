//! Various Leptos widgets

use std::{fmt::Display, hash::Hash, str::FromStr};

use cfg_if::cfg_if;
#[cfg(feature = "ssr")]
use http::status::StatusCode;
use leptos::*;
#[cfg(feature = "ssr")]
use leptos_axum::ResponseOptions;
use leptos_router::*;

// A loading spinner
#[component]
pub fn Spinner() -> impl IntoView {
    view! {
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
pub fn Link(link: &'static str, text: &'static str) -> impl IntoView {
    view! {
        <A href=link class="text-primary-100 hover:no-underline">
            {text}
        </A>
    }
}

/// A `<a>` link that links to an external site
#[component]
pub fn LinkExternal(link: &'static str, text: &'static str) -> impl IntoView {
    view! {
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
pub fn NotFound() -> impl IntoView {
    cfg_if! { if #[cfg(feature="ssr")] {
        if let Some(response) = use_context::<ResponseOptions>() {
            response.set_status(StatusCode::NOT_FOUND);
        }
    }}
    view! {
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
pub fn Errors(errors: Errors) -> impl IntoView {
    tracing::error!("Errors: {:?}", errors);
    view! {
        <div class="flex flex-col justify-center overflow-auto">
            <header class="p-2 text-xl font-bold text-white bg-error-500">"💣 ERROR 💣"</header>
            <div class="p-2 font-mono text-sm text-left whitespace-pre-wrap bg-black">
                <ul>
                    {errors
                        .into_iter()
                        .map(|(k, e)| {
                            view! {
                                <li class="mb-4">
                                    <header class="px-2 mb-2 font-bold text-gray-100">
                                        {format!("{:?}", k)}
                                    </header>
                                    <div class="px-2 text-gray-400 hover:text-gray-100">
                                        {e.to_string()}
                                    </div>
                                </li>
                            }
                        })
                        .collect_view()}
                </ul>
            </div>
        </div>
    }
}

/// Like [Suspense] but also handles errors using [ErrorBoundary]
#[component(transparent)]
pub fn SuspenseWithErrorHandling(children: ChildrenFn) -> impl IntoView {
    let children = store_value(children);
    view! {
        <Suspense fallback=move || view! { <Spinner/> }>
            <ErrorBoundary fallback=|errors| {
                view! { <Errors errors=errors.get()/> }
            }>{children.with_value(|c| c())}</ErrorBoundary>
        </Suspense>
    }
}

/// An input element component with suggestions.
///
/// A label, input element, and datalist are rendered, as well as error div.
/// [FromStr::from_str] is used to parse the input value into `K`.
///
/// Arguments:
/// * `id`: The id of the input element
/// * `label`: The label string
/// * `suggestions`: The initial suggestions to show in the datalist
/// * `val`: The [RwSignal] mirror'ing the input element value
#[component]
pub fn TextInput<K>(
    id: &'static str,
    label: &'static str,
    /// Initial suggestions to show in the datalist
    suggestions: Vec<K>,
    val: RwSignal<K>,
) -> impl IntoView
where
    K: ToString + FromStr + Hash + Eq + Clone + Display + 'static,
    <K as std::str::FromStr>::Err: Display,
{
    let datalist_id = &format!("{}-datalist", id);
    // Input query to the server fn
    // Errors in input element (based on [FromStr::from_str])
    let (input_err, set_input_err) = create_signal(None::<String>);
    view! {
        <label for=id>{label}</label>
        <input
            list=datalist_id
            id=id.to_string()
            type="text"
            class="w-full p-1 font-mono"
            on:change=move |ev| {
                match FromStr::from_str(&event_target_value(&ev)) {
                    Ok(s) => {
                        val.set(s);
                        set_input_err(None)
                    }
                    Err(e) => set_input_err(Some(e.to_string())),
                }
            }

            prop:value=move || val.get().to_string()
        />
        <span class="text-red-500">{input_err}</span>
        // TODO: use local storage, and cache user's inputs
        <datalist id=datalist_id>
            {suggestions
                .iter()
                .map(|s| view! { <option value=s.to_string()></option> })
                .collect_view()}

        </datalist>
    }
}
