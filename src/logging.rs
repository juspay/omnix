//! Logging setup for the server and client

#[cfg(feature = "ssr")]
use tower_http::{
    classify::{ServerErrorsAsFailures, SharedClassifier},
    trace::TraceLayer,
};
use tracing_subscriber::{filter::Directive, EnvFilter};

/// Setup server-side logging using [tracing_subscriber]
#[cfg(feature = "ssr")]
pub fn setup_server_logging(log_directives: Vec<Directive>) {
    let filter = log_directives.iter().fold(
        EnvFilter::from_env("NIX_BROWSER_LOG"),
        |filter, directive| filter.add_directive(directive.clone()),
    );
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
        .make_span_with(trace::DefaultMakeSpan::new().level(Level::DEBUG))
        .on_response(trace::DefaultOnResponse::new().level(Level::DEBUG))
}
