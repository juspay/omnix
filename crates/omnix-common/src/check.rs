//! Prerequisite checks for the Omnix project.

use std::path::PathBuf;
use which::{which, Error};

/// Check if Nix is installed.
pub fn nix_installed() -> bool {
    which_strict("nix").is_some()
}

/// Check if a binary is available in the system's PATH and return its path.
/// Returns None if the binary is not found, panics on unexpected errors.
pub fn which_strict(binary: &str) -> Option<PathBuf> {
    match which(binary) {
        Ok(path) => Some(path),
        Err(Error::CannotFindBinaryPath) => None,
        Err(e) => panic!(
            "Unexpected error while searching for binary '{}': {:?}",
            binary, e
        ),
    }
}
