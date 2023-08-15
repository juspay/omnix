#![feature(associated_type_defaults)]
//! nix-browser crate; see GitHub [README] for details.
//!
//! [README]: https://github.com/juspay/nix-browser
pub mod app;
pub mod nix;
pub mod query;
#[cfg(feature = "hydrate")]
use wasm_bindgen::prelude::wasm_bindgen;
#[cfg(feature = "ssr")]
pub mod command;
#[cfg(feature = "ssr")]
pub mod server;

/// Main entry point for the WASM frontend
#[cfg(feature = "hydrate")]
#[wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    use leptos::*;

    setup_logging();
    tracing::info!("Hydrating app");
    leptos::mount_to_body(move |cx| {
        view! { cx, <App/> }
    });
}

/// Setup browser console logging using [tracing_subscriber_wasm]
#[cfg(feature = "hydrate")]
fn setup_logging() {
    tracing_subscriber::fmt()
        .with_writer(
            // To avoide trace events in the browser from showing their
            // JS backtrace, which is very annoying, in my opinion
            tracing_subscriber_wasm::MakeConsoleWriter::default()
                .map_trace_level_to(tracing::Level::DEBUG),
        )
        .with_max_level(tracing::Level::DEBUG)
        // For some reason, if we don't do this in the browser, we get
        // a runtime error.
        .without_time()
        .init();
}
