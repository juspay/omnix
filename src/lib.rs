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
    // initializes logging using the `log` crate
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    leptos::mount_to_body(move |cx| {
        view! { cx, <App/> }
    });
}
