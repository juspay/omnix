# Equivalent to passing --no-open
export NIX_BROWSER_NO_OPEN := "true"

default:
    @just --list

# Auto-format the source tree
fmt:
    treefmt

# Run the project locally
watch $RUST_BACKTRACE="1":
    cargo leptos watch

# Run cargo in release mode (prints red panic)
watch-release:
    cargo leptos watch --release

# Run tests (backend & frontend)
test:
    cargo-test

doc:
    cargo-doc-live
