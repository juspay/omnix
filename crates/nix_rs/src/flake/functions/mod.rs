//! Calling Nix functions (defined in a flake) from Rust, as if to provide FFI.
//
// This model provides a simpler alternative to Flake Schemas, but it can also do more than Flake Schemas can (such as building derivations).

pub mod core;
pub mod metadata;
