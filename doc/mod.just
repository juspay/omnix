default:
  @just --list doc

# Run mdbook live server
run:
  mdbook serve

# Do link checks on docs
check:
    nix run .#doc-linkCheck

# Run cargo docs server (live reloading)
[no-cd]
cargo:
    cargo-doc-live
