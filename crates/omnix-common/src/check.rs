//! Prerequisite checks for the Omnix project.

use which::{which, Error};

/// Check if Nix is installed.
pub fn nix_installed() -> bool {
    match which("nix") {
        Ok(_) => true,
        Err(Error::CannotFindBinaryPath) => false,
        Err(e) => panic!("Unexpected error while searching for Nix: {:?}", e),
    }
}
