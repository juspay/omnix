//! omnix-ci: CI for Nix projects
#![warn(missing_docs)]
#![feature(exit_status_error)]
#![feature(let_chains)]
pub mod command;
pub mod config;
pub mod flake_ref;
pub mod github;
pub mod nix;
pub mod step;
