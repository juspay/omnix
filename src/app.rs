//! Frontend UI entry point
use crate::nix::health::*;
use crate::nix::info::get_nix_info;
use cfg_if::cfg_if;
#[cfg(feature = "ssr")]
use http::status::StatusCode;
use leptos::*;
#[cfg(feature = "ssr")]
use leptos_axum::ResponseOptions;
use leptos_meta::*;
use leptos_router::*;

/// Main frontend application container
#[component]
pub fn App(cx: Scope) -> impl IntoView {
    provide_meta_context(cx);

    view! { cx,
        <Stylesheet id="leptos" href="/pkg/nix-browser.css"/>
        <Router fallback=|cx| {
            view! { cx, <NotFound/> }
        }>
            <Title formatter=|s| format!("{s} - nix-browser")/>
            <div class="flex justify-center w-full min-h-screen bg-center bg-cover bg-base-200">
                <div class="container flex flex-col items-center mx-auto max-w-prose">
                    <Nav/>
                    <div class="z-0 flex col-start-1 row-start-1 px-2 text-center">
                        <div class="flex flex-col space-y-3">
                            <Routes>
                                <Route path="" view=Dashboard/>
                                <Route path="/health" view=NixHealth/>
                                <Route path="/info" view=NixInfo/>
                                <Route path="/about" view=About/>
                            </Routes>
                        </div>
                    </div>
                </div>
            </div>
        </Router>
    }
}

/// Navigation bar
#[component]
fn Nav(cx: Scope) -> impl IntoView {
    let class = "px-3 py-2";
    view! { cx,
        <nav class="flex flex-row w-full mb-3 text-white md:rounded-b bg-primary-800">
            <A exact=true href="/" class=class>
                "Dashboard"
            </A>
            <A exact=true href="/health" class=class>
                "Nix Health"
            </A>
            <A exact=true href="/info" class=class>
                "Nix Info"
            </A>
            <A exact=true href="/about" class=class>
                "About"
            </A>
            <div class=format!("flex-grow font-bold text-end {}", class)>"🌍 nix-browser"</div>
        </nav>
    }
}

/// Home page
#[component]
fn Dashboard(cx: Scope) -> impl IntoView {
    let health_check = create_resource(cx, move || (), move |_| get_health_checks());
    // A Card component
    #[component]
    fn Card(cx: Scope, href: &'static str, children: Children) -> impl IntoView {
        view! { cx,
            <A
                href=href
                class="flex items-center justify-center w-64 h-48 p-2 m-2 rounded-lg bg-secondary-100 hover:bg-secondary-200"
            >
                <span class="text-4xl text-base-800">{children(cx)}</span>
            </A>
        }
    }
    tracing::debug!("Rendering Dashboard page");
    view! { cx,
        <Title text="Dashboard"/>
        <h1 class="text-5xl font-bold">"Dashboard"</h1>
        <div id="cards" class="flex flex-row">
            // TODO: This should show green or red depending on the health check status
            <Card href="/health">
                "Nix Health Check " <Suspense fallback=move || view! { cx, <Spinner/> }>
                    <ErrorBoundary fallback=|cx, errors| {
                        view! { cx, <Errors errors=errors.get()/> }
                    }>
                        {move || {
                            health_check
                                .read(cx)
                                .map(|r| {
                                    r
                                        .map(|v| {
                                            if v.is_healthy() {
                                                view! { cx, <span class="text-green-500">{"✓"}</span> }
                                                    .into_view(cx)
                                            } else {
                                                view! { cx, <span class="text-red-500">{"✗"}</span> }
                                                    .into_view(cx)
                                            }
                                        })
                                })
                        }}

                    </ErrorBoundary>
                </Suspense>

            </Card>
            <Card href="/info">"Nix Info ℹ️"</Card>
        </div>
    }
}

/// Nix information
#[component]
fn NixInfo(cx: Scope) -> impl IntoView {
    let nix_info = create_resource(cx, move || (), move |_| get_nix_info());
    let title = "Nix Info";
    view! { cx,
        <Title text=title/>
        <h1 class="text-5xl font-bold">{title}</h1>
        <Suspense fallback=move || view! { cx, <Spinner/> }>
            <ErrorBoundary fallback=|cx, errors| view! { cx, <Errors errors=errors.get()/> }>
                <div class="my-1 text-left">{move || nix_info.read(cx)}</div>
            </ErrorBoundary>
        </Suspense>
    }
}

/// Nix health checks
#[component]
fn NixHealth(cx: Scope) -> impl IntoView {
    let health_check = create_resource(cx, move || (), move |_| get_health_checks());
    let title = "Nix Health";
    view! { cx,
        <Title text=title/>
        <h1 class="text-5xl font-bold">{title}</h1>
        <Suspense fallback=move || view! { cx, <Spinner/> }>
            <ErrorBoundary fallback=|cx, errors| view! { cx, <Errors errors=errors.get()/> }>
                <div class="my-1 text-left">{move || health_check.read(cx)}</div>
            </ErrorBoundary>
        </Suspense>
    }
}

/// About page
#[component]
fn About(cx: Scope) -> impl IntoView {
    view! { cx,
        <Title text="About"/>
        <h1 class="text-5xl font-bold">"About"</h1>
        <p>
            "nix-browser is still work in progress. Track its development "
            <LinkExternal link="https://github.com/juspay/nix-browser" text="on Github"/>
        </p>
    }
}

// A loading spinner
#[component]
fn Spinner(cx: Scope) -> impl IntoView {
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

/// <a> link
#[component]
fn Link(cx: Scope, link: &'static str, text: &'static str) -> impl IntoView {
    view! { cx,
        <A href=link class="text-primary-100 hover:no-underline">
            {text}
        </A>
    }
}

#[component]
fn LinkExternal(cx: Scope, link: &'static str, text: &'static str) -> impl IntoView {
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
fn NotFound(cx: Scope) -> impl IntoView {
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
fn Errors(cx: Scope, errors: Errors) -> impl IntoView {
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
