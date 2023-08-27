#![feature(associated_type_defaults)]
//! nix-browser crate; see GitHub [README] for details.
//!
//! [README]: https://github.com/juspay/nix-browser
pub mod app;
pub mod leptos_extra;
pub mod logging;
pub mod nix;
pub mod widget;
#[cfg(feature = "hydrate")]
use wasm_bindgen::prelude::wasm_bindgen;
#[cfg(feature = "ssr")]
pub mod cli;
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

    logging::setup_client_logging();
    tracing::info!("Hydrating app");
    leptos::mount_to_body(move |cx| {
        view! { cx, <App/> }
    });
}
