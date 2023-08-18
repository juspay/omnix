//! Axum server
use std::convert::Infallible;

use crate::app::App;
use axum::response::Response as AxumResponse;
use axum::routing::IntoMakeService;
use axum::{body::Body, http::Request, response::IntoResponse};
use axum::{routing::post, Router};
use hyper::server::conn::AddrIncoming;
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tower_http::trace::{self, TraceLayer};
use tracing::instrument;

use crate::cli;

/// Axum server main entry point
pub async fn main(args: cli::Args) {
    setup_logging(args.log_level());
    let server = create_server(args.log_level()).await;
    if !args.no_open {
        open_http_app(server.local_addr()).await;
    }
    server.await.unwrap()
}

/// Create an Axum server for the Leptos app
#[instrument(name = "server")]
#[allow(clippy::async_yields_async)]
async fn create_server(
    trace_level: tracing::Level,
) -> axum::Server<AddrIncoming, IntoMakeService<axum::Router>> {
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
                .make_span_with(trace::DefaultMakeSpan::new().level(trace_level))
                .on_response(trace::DefaultOnResponse::new().level(trace_level)),
        )
        .with_state(conf.leptos_options.clone());

    let server = axum::Server::bind(&conf.leptos_options.site_addr).serve(app.into_make_service());
    tracing::info!("App is running at http://{}", server.local_addr());
    server
}

fn setup_logging(trace_level: tracing::Level) {
    tracing_subscriber::fmt()
        .with_max_level(trace_level)
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

/// Open a http address in the user's web browser
async fn open_http_app(addr: SocketAddr) {
    let url = format!("http://{}", &addr);
    if let Err(err) = open::that(url) {
        tracing::warn!("Unable to open in web browser: {}", err)
    }
}
