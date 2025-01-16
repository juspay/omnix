## Rust + Nix FFI

https://github.com/srid/devour-flake introduced the idea of defining "functions" in Nix flake, that can be called from any external process. The flakes `inputs` acts as "arguments" taken by this function, with the flake's package output acting as its `output`. 

This package, `nix_rs::flake::functions`, provides the Rust FFI adapter to work with such Nix functions in Rust, using simpler API.

In effect, this generalizes devour-flake to be able to define such functions. See [`devour_flake.rs`] for an example.
