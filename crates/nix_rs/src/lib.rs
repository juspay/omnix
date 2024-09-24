//! Rust crate to interact with Nix
//!
//! This crate exposes various types representing what nix command gives us,
//! along with a `from_nix` command to evaluate them.
#![warn(missing_docs)]
pub mod command;
pub mod config;
pub mod copy;
pub mod detsys_installer;
pub mod env;
pub mod flake;
pub mod info;
pub mod refs;
pub mod store;
pub mod system_list;
pub mod version;
