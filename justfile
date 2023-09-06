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

# Run 'cargo run' for nix-health CLI in watch mode
watch-nix-health:
    cargo watch -- cargo run --bin nix-health --features=ssr

# Run tests (backend & frontend)
test:
    cargo test
    cargo leptos test

# Run docs server (live reloading)
doc:
    cargo-doc-live

# Run CI locally
ci:
    nixci
