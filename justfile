export NIX_BROWSER_NO_OPEN := "true"

default:
    @just --list 

# Auto-format the source tree
fmt:
    treefmt

alias f := fmt

# Run the project locally
watch $RUST_BACKTRACE="1":
    dx serve --hot-reload

alias w := watch

# Run 'cargo run' for nix-health CLI in watch mode. Example: just watch-nix-health github:nammayatri/nammayatri
watch-nix-health *ARGS:
    cargo watch -- cargo run --bin nix-health --features=ssr -- {{ARGS}}

alias wh := watch-nix-health

# Run tests (backend & frontend)
test:
    cargo test
    cargo leptos test

# Run end-to-end tests against release server
e2e-release:
    nix run .#e2e-playwright-test

# Run end-to-end tests against `just watch` server
e2e:
    cd e2e && TEST_PORT=3000 playwright test --project chromium

# Run docs server (live reloading)
doc:
    cargo-doc-live

# Run CI locally
ci:
    nixci

# Setup node_modules using Nix (invoked automatically by nix-shell)
node_modules NODE_PATH:
    rm -f ./e2e/node_modules
    ln -sf ${NODE_PATH} ./e2e/node_modules
