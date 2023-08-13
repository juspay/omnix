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
