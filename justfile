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
    echo "Docs will be available at ➡️ http://localhost:8008/nix_browser/"
    cargo watch -s 'cargo doc --document-private-items --all-features && browser-sync start --port 8008 --ss target/doc -s target/doc --directory --no-open'
