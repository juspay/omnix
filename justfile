# Equivalent to passing --no-open
export NIX_BROWSER_NO_OPEN := "true"
export RUST_BACKTRACE := "1"

default:
    @just --list

# Auto-format the source tree
fmt:
    treefmt

# Run the project locally
watch *ARGS:
    cargo leptos watch

# Run tests (backend & frontend)
test:
    cargo watch -- cargo leptos test

doc:
    cargo-doc-live
