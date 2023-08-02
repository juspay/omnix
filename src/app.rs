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

#[component]
fn Home(cx: Scope) -> impl IntoView {
    view! { cx,
        <div class="flex flex-col items-center justify-center min-h-screen bg-blue-300">
            <div class="flex flex-col items-center justify-start px-4 py-8 mx-auto bg-white border-4 rounded-lg">
                <Header1 text="Welcome to nix-browser" />
                <div class="items-left">
                    <Header2 text="Nix Info" />
                    <p class="my-1"><pre>
                        <Await
                            future=|_| nix::nix_info()
                            bind:data
                        >
                            {format!("{data:?}")}
                        </Await>
                    </pre></p>
                </div>
            </div>
        </div>
    }
}

#[component]
fn Link(
    cx: Scope,
    link: &'static str,
    text: &'static str,
    #[prop(optional)] rel: Option<&'static str>,
) -> impl IntoView {
    view! {cx,
        <a href=link class="text-red-500 underline hover:no-underline" rel=rel>{text}</a>
    }
}

#[component]
fn Header1(cx: Scope, text: &'static str) -> impl IntoView {
    view! {cx,
        <h1 class="my-3 text-3xl font-bold">{text}</h1>
    }
}
#[component]
fn Header2(cx: Scope, text: &'static str) -> impl IntoView {
    view! {cx,
        <h2 class="my-2 text-2xl font-bold text-gray-600">{text}</h2>
    }
}

#[component]
fn NotFound(cx: Scope) -> impl IntoView {
    view! { cx,
        <div class="flex flex-row justify-center text-3xl text-red-500">
            "404: Page not found"
        </div>
    }
}
