//! Axum server
use std::convert::Infallible;

use crate::app::App;
use axum::response::Response as AxumResponse;
use axum::routing::IntoMakeService;
use axum::{body::Body, http::Request, response::IntoResponse, Json};
use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
};
use hyper::server::conn::AddrIncoming;
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use nix_rs::info::NixInfo;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tracing::instrument;

use crate::cli;

/// Axum server main entry point
pub async fn main(args: cli::Args) {
    crate::logging::setup_server_logging(&args.verbosity);
    let leptos_options = get_leptos_options(&args).await;
    let server = create_server(leptos_options).await;
    if !args.no_open {
        open_http_app(server.local_addr()).await;
    }
    server.await.unwrap()
}

/// Create an Axum server for the Leptos app
#[instrument(name = "server")]
#[allow(clippy::async_yields_async)]
async fn create_server(
    leptos_options: leptos_config::LeptosOptions,
) -> axum::Server<AddrIncoming, IntoMakeService<axum::Router>> {
    tracing::debug!("Firing up Leptos app with config: {:?}", leptos_options);
    leptos_query::suppress_query_load(true); // https://github.com/nicoburniske/leptos_query/issues/6
    let routes = generate_route_list(|cx| view! { cx, <App/> }).await;
    leptos_query::suppress_query_load(false);
    let client_dist = ServeDir::new(leptos_options.site_root.clone());
    let leptos_options_clone = leptos_options.clone(); // A copy to move to the closure below.
    let not_found_service =
        tower::service_fn(move |req| not_found_handler(leptos_options_clone.to_owned(), req));
    let app = Router::new()
        // data API routes
        .route("/api/data/nix-info", get(get_nix_info_handler))
        // server functions API routes
        .route("/api/*fn_name", post(leptos_axum::handle_server_fns))
        // application routes
        .leptos_routes(&leptos_options, routes, |cx| view! { cx, <App/> })
        // static files are served as fallback (but *before* falling back to
        // error handler)
        .fallback_service(client_dist.clone().not_found_service(not_found_service))
        // enable HTTP request logging
        .layer(crate::logging::http_trace_layer())
        .with_state(leptos_options.clone());

    let server = axum::Server::bind(&leptos_options.site_addr).serve(app.into_make_service());
    tracing::info!("nix-browser web 🌀️ http://{}", server.local_addr());
    server
}

async fn get_leptos_options(args: &cli::Args) -> leptos_config::LeptosOptions {
    let conf_file = get_configuration(None).await.unwrap();
    leptos_config::LeptosOptions {
        site_addr: args.site_addr.unwrap_or(conf_file.leptos_options.site_addr),
        ..conf_file.leptos_options
    }
}

/// Handler for nix-info data
async fn get_nix_info_handler() -> Result<impl IntoResponse, StatusCode> {
    let v = NixInfo::from_nix(&nix_rs::command::NixCmd::default()).await;
    match v {
        Ok(info) => Ok(Json(info)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
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
