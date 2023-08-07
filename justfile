default:
    @just --list

# Auto-format the source tree
fmt:
    treefmt

# Run the project locally
watch *ARGS:
    cargo leptos watch

# Run tests (backend)
test:
    cargo watch -x test --features=ssr
