//! Logging setup for the server and client

#[cfg(feature = "ssr")]
use tower_http::{
    classify::{ServerErrorsAsFailures, SharedClassifier},
    trace::TraceLayer,
};
#[cfg(feature = "ssr")]
use tracing_subscriber::filter::{Directive, LevelFilter};
#[cfg(feature = "ssr")]
use tracing_subscriber::EnvFilter;

/// Setup server-side logging using [tracing_subscriber]
#[cfg(feature = "ssr")]
pub fn setup_server_logging(filter: EnvFilter) {
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .compact()
        .init();
}

/// Setup browser console logging using [tracing_subscriber_wasm]
#[cfg(feature = "hydrate")]
pub fn setup_client_logging() {
    tracing_subscriber::fmt()
        .with_writer(
            // To avoide trace events in the browser from showing their
            // JS backtrace, which is very annoying, in my opinion
            tracing_subscriber_wasm::MakeConsoleWriter::default()
                .map_trace_level_to(tracing::Level::DEBUG),
        )
        .with_max_level(tracing::Level::INFO)
        // For some reason, if we don't do this in the browser, we get
        // a runtime error.
        .without_time()
        .init();
}

/// Setup HTTP request logging
#[cfg(feature = "ssr")]
pub fn http_trace_layer() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>> {
    use tower_http::trace;
    use tracing::Level;

    TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(Level::INFO))
}

/// How to handle user's use of multiple `-v`
///
/// Convert user's `-v` invocation to [tracing] log level directives
#[cfg(feature = "ssr")]
pub fn log_directives_for_verbosity(verbose_level: u8) -> Vec<Directive> {
    // Allow warnings+errors from all crates.
    let base_filter = LevelFilter::WARN.into();
    match verbose_level {
        // Default
        0 => vec![base_filter, "nix_browser=info".parse().unwrap()],
        // -v: log app DEBUG level, as well as http requests
        1 => vec![
            base_filter,
            "nix_browser=debug".parse().unwrap(),
            "tower_http=info".parse().unwrap(),
        ],
        // -vv: log app TRACE level, as well as http requests
        2 => vec![
            base_filter,
            "nix_browser=trace".parse().unwrap(),
            "tower_http=info".parse().unwrap(),
        ],
        // -vvv: log DEBUG level of app and libraries
        3 => vec![LevelFilter::DEBUG.into()],
        // -vvvv: log TRACE level of app and libraries
        _ => vec![LevelFilter::TRACE.into()],
    }
}
