default:
  @just --list doc

# Run mdbook live server
run:
  nix run .#serve

# Do link checks on docs
check:
    nix run .#linkCheck

# Run cargo docs server (live reloading)
[no-cd]
cargo:
    cargo-doc-live
