//! Prerequisite checks for the Omnix project.

use which::which;

/// Check if Nix is installed.
pub fn nix_installed() -> bool {
    let out = which("nix");
    out.is_ok()
}
