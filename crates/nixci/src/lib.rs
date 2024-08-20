//! nixci: CI for Nix projects
#![warn(missing_docs)]
#![feature(exit_status_error)]
pub mod command;
pub mod config;
pub mod flake_ref;
pub mod github;
pub mod nix;
pub mod step;
