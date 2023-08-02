use crate::thing::{read_things, ReadThings, Thing};
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
    let thing = Thing::new("Hello from frontend".to_string());
    let things = create_local_resource(cx, move || (), move |_| read_things());
    view! { cx,
        <div class="flex flex-col items-center justify-center min-h-screen bg-blue-300">
            <div class="flex flex-col items-center justify-start px-4 py-8 mx-auto bg-white border-4 rounded-lg">
                <Header1 text="Welcome to nix-browser placeholder" />
                <div class="items-left">
                    <Header2 text="Frontend" />
                    <p class="my-1">"This value ⤵️ is generated in-browser:"</p>
                    <pre>{thing.browser_view()}</pre>
                    <Header2 text="Backend" />
                    <pre>fn_url: {ReadThings::url()}</pre>
                    {move || {
                        things.read(cx)
                            .map(move |things| {
                                log!("things: {:?}", things);
                                match things {
                                    Err(e) => {
                                        view! { cx, <pre class="p-2 my-2 font-bold bg-red-200 shadow-lg">"Server Error: " {e.to_string()}</pre>}.into_view(cx)
                                    }
                                    Ok(things) => {
                                        things.into_iter().map(move |thing| {
                                            view! {
                                                cx,
                                                <li>{thing.browser_view()}</li>
                                            }
                                        }).collect_view(cx)
                                    }
                            }
                        })
                    }}
                    <Link link="/hello" text="request backend /hello API" rel="external" />
                    <div><Link link="/sdf" text="broken link" /></div>
                    <Counter />
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

/// Renders the home page of your application.
#[component]
fn Counter(cx: Scope) -> impl IntoView {
    // Creates a reactive value to update the button
    let (count, set_count) = create_signal(cx, 0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! { cx,
        <div class="mx-auto my-8 text-center md:container">
            <Header2 text="Leptops Counter" />
            <button
                class="p-4 border-2 rounded-full shadow-lg active:shadow-none bg-blue-50 hover:bg-blue-200 active:bg-blue-500"
                on:click=on_click>"Click Me: " {count}</button>
        </div>
    }
}
