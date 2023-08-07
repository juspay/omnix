use crate::nix::info::get_nix_info;
use cfg_if::cfg_if;
#[cfg(feature = "ssr")]
use http::status::StatusCode;
use leptos::*;
#[cfg(feature = "ssr")]
use leptos_axum::ResponseOptions;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    provide_meta_context(cx);

    view! {
        cx,
        <Stylesheet id="leptos" href="/pkg/nix-browser.css"/>
        <Router fallback=|cx| {
            cfg_if! { if #[cfg(feature="ssr")] {
                if let Some(response) = use_context::<ResponseOptions>(cx) {
                    response.set_status(StatusCode::NOT_FOUND);
                }
            }}
            view! { cx, <NotFound /> }.into_view(cx)
        }>
            <Routes>
                <Route path="" view=  move |cx| view! { cx, <Home/> }/>
            </Routes>
        </Router>
    }
}

/// Home page
#[component]
fn Home(cx: Scope) -> impl IntoView {
    let nix_info = create_resource(cx, || (), |_| async { get_nix_info().await });
    view! { cx,
        <div class="grid w-full min-h-screen bg-center bg-cover bg-base-200 place-items-center">
            <div class="z-0 flex items-center justify-center col-start-1 row-start-1 text-center">
              <div class="flex flex-col space-y-3">
                <h1 class="text-5xl font-bold">Welcome to nix-browser</h1>
                <p class="py-6">
                    <h2 class="text-3xl font-bold text-gray-500">"Nix Info"</h2>
                    <div class="my-1">
                        <Suspense fallback=move || view! {cx, <p>"Loading nix-info"</p> }>
                            <ErrorBoundary fallback=|cx, errors| view! { cx, <Errors errors=errors.get() /> } >
                                {nix_info.read(cx)}
                            </ErrorBoundary>
                        </Suspense>
                    </div>
                </p>
                <Link link="https://github.com/juspay/nix-browser" text="Source Code" rel="external" />
              </div>
            </div>
        </div>
    }
}

/// <a> link
#[component]
fn Link(
    cx: Scope,
    link: &'static str,
    text: &'static str,
    #[prop(optional)] rel: Option<&'static str>,
) -> impl IntoView {
    view! {cx,
        <a href=link class="underline text-primary-500 hover:no-underline" rel=rel>{text}</a>
    }
}

/// 404 page
#[component]
fn NotFound(cx: Scope) -> impl IntoView {
    view! {cx,
        // The HTML for 404 not found
        <div class="grid w-full min-h-screen bg-center bg-cover bg-base-200 place-items-center">
            <div class="z-0 flex items-center justify-center col-start-1 row-start-1 text-center">
              <div class="flex flex-col space-y-3">
                <h1 class="text-5xl font-bold">404</h1>
                <p class="py-6">
                    <h2 class="text-3xl font-bold text-gray-500">"Page not found"</h2>
                    <p class="my-1">The page you are looking for does not exist.</p>
                </p>
                <Link link="/" text="Go to home page" />
              </div>
            </div>
        </div>
    }
}

/// Display errors to the user
#[component]
fn Errors(cx: Scope, errors: Errors) -> impl IntoView {
    view! { cx,
        <div class="flex flex-row justify-center overflow-auto text-xl text-white bg-error-500">
            <div class="font-mono whitespace-pre-wrap">
                <ul>
                    {errors
                        .into_iter()
                        .map(|(_, e)| view! { cx, <li>{e.to_string()}</li>})
                        .collect_view(cx)
                    }
                </ul>
            </div>
        </div>
    }
}
