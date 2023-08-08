//! nix-browser crate; see GitHub [README] for details.
//!
//! [README]: https://github.com/juspay/nix-browser
pub mod app;
pub mod nix;
#[cfg(feature = "hydrate")]
use wasm_bindgen::prelude::wasm_bindgen;
#[cfg(feature = "ssr")]
pub mod server;

/// Main entry point for the WASM frontend
#[cfg(feature = "hydrate")]
#[wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    use leptos::*;
    use tracing_subscriber::fmt;
    use tracing_subscriber_wasm::MakeConsoleWriter;
    // initializes logging using the `log` crate
    // _ = console_log::init_with_level(log::Level::Debug);
    // console_error_panic_hook::set_once();

    fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(
            // To avoide trace events in the browser from showing their
            // JS backtrace, which is very annoying, in my opinion
            MakeConsoleWriter::default().map_trace_level_to(tracing::Level::DEBUG),
        )
        // For some reason, if we don't do this in the browser, we get
        // a runtime error.
        .without_time()
        .init();

    leptos::mount_to_body(move |cx| {
        view! { cx, <App/> }
    });
}
