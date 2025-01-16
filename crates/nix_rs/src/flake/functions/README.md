## Rust + Nix FFI

https://github.com/srid/devour-flake introduced the idea of defining "functions" in Nix flake, that can be called from any external process. The flake's package derivation acts as the function "body", with its `inputs` acting as function "arguments"; the built output of that derivation is the function's "output".

This Rust package, `nix_rs::flake::functions`, provides the Rust FFI adapter to work with such Nix functions in Rust, using simpler API. You define your input & output structs in Rust, implement the `FlakeFn` trait and voilà !

In effect, this generalizes `devour-flake` to be able to define such functions. See `devour_flake.rs` in this repo for an example.

## Inspiration

- [devour-flake](https://github.com/srid/devour-flake): Original use of this pattern.
- [inspect](https://github.com/DeterminateSystems/inspect) works similar to `devour-flake`, but is tied to flake schemas, and the function body is hardcoded (just as `devour-flake`).
