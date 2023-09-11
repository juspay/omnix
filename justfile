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
