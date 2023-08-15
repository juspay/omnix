//! Axum server
use std::convert::Infallible;

use crate::app::App;
use axum::response::Response as AxumResponse;
use axum::{body::Body, http::Request, response::IntoResponse};
use axum::{routing::post, Router};
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use tower_http::services::ServeDir;
use tower_http::trace::{self, TraceLayer};
use tracing::{instrument, Level};

use crate::cli;

/// Axum server main entry point
pub async fn main(args: cli::Args) {
    setup_logging();
    run_server(args).await
}

#[instrument(name = "server")]
async fn run_server(args: cli::Args) {
    let conf = get_configuration(None).await.unwrap();
    tracing::debug!("Firing up Leptos app with config: {:?}", conf);
    leptos_query::suppress_query_load(true); // https://github.com/nicoburniske/leptos_query/issues/6
    let routes = generate_route_list(|cx| view! { cx, <App/> }).await;
    leptos_query::suppress_query_load(false);
    let client_dist = ServeDir::new(conf.leptos_options.site_root.clone());
    let leptos_options = conf.leptos_options.clone(); // A copy to move to the closure below.
    let not_found_service =
        tower::service_fn(move |req| not_found_handler(leptos_options.to_owned(), req));
    let app = Router::new()
        // server functions API routes
        .route("/api/*fn_name", post(leptos_axum::handle_server_fns))
        // application routes
        .leptos_routes(&conf.leptos_options, routes, |cx| view! { cx, <App/> })
        // static files are served as fallback (but *before* falling back to
        // error handler)
        .fallback_service(client_dist.clone().not_found_service(not_found_service))
        // enable HTTP request logging
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(conf.leptos_options.clone());
    let server = axum::Server::bind(&conf.leptos_options.site_addr).serve(app.into_make_service());
    tracing::info!("App is running at http://{}", server.local_addr());
    let url = format!("http://{}", &server.local_addr());
    if !args.no_open {
        if let Err(err) = open::that(&url) {
            tracing::warn!("Unable to open in web browser: {}", err)
        }
    }
    server.await.unwrap()
}

fn setup_logging() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO) // TODO: --verbose should use DEBUG
        .compact()
        .init();
}

/// Handler for missing routes
///
/// On missing routes, just delegate to the leptos app, which has a route
/// fallback rendering 404 response.
async fn not_found_handler(
    options: LeptosOptions,
    req: Request<Body>,
) -> Result<AxumResponse, Infallible> {
    let handler =
        leptos_axum::render_app_to_stream(options.to_owned(), move |cx| view! { cx, <App/> });
    Ok(handler(req).await.into_response())
}
