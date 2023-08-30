export NIX_BROWSER_NO_OPEN := "true"

default:
    @just --list 

# Auto-format the source tree
fmt:
    treefmt

alias f := fmt

# Run the project locally
watch $RUST_BACKTRACE="1":
    cargo leptos watch

alias w := watch

# Run tests (backend & frontend)
test:
    cargo-test

# Run tests (e2e)
e2e-test:
    nix run .#cargo-e2e-test

# Run docs server (live reloading)
doc:
    cargo-doc-live

# Run CI locally
ci:
    nixci
