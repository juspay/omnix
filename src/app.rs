use crate::nix;
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
    view! { cx,
        <div class="grid w-full min-h-screen bg-center bg-cover bg-base-200 place-items-center">
            <div class="z-0 flex items-center justify-center col-start-1 row-start-1 text-center">
              <div class="flex flex-col space-y-3">
                <h1 class="text-5xl font-bold">Welcome to nix-browser</h1>
                <p class="py-6">
                    <h2 class="text-3xl font-bold text-gray-500">"Nix Info"</h2>
                    <p class="my-1"><pre>
                        <Await
                            future=|_| nix::nix_info()
                            bind:data
                        >
                            {format!("{data:?}")}
                        </Await>
                    </pre></p>
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
    view! { cx,
        <div class="flex flex-row justify-center text-3xl text-error-500">
            "404: Page not found"
        </div>
    }
}
