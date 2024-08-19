//! Prerequisite checks for the Omnix project.

use which::which;

pub fn nix_installed() -> bool {
    let out = which("nix");
    out.is_ok()
}
