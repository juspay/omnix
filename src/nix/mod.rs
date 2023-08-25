//! Rust module to interact with Nix
#[cfg(feature = "ssr")]
pub mod command;
pub mod config;
pub mod flake;
pub mod health;
pub mod info;
pub mod refs;
pub mod version;
