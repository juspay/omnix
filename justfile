default:
    @just --list

# Auto-format the source tree
fmt:
    # Due a bug in dx fmt, we cannot run it on files using macros
    find crates/omnix-gui/src/app/ -name \*.rs | grep -v state.rs | grep -v state/ | xargs -n1 sh -c 'echo "📔 $1"; dx fmt -f $1' sh
    # Run treefmt *after* 'dx fmt' because the latter rewrites the former!
    treefmt

alias f := fmt

# CI=true for https://github.com/tauri-apps/tauri/issues/3055#issuecomment-1624389208)
bundle $CI="true":
    # HACK (change PWD): Until https://github.com/DioxusLabs/dioxus/issues/1283
    cd ./crates/omnix-gui/assets && dx bundle --release
    nix run nixpkgs#lsd -- --tree ./dist/bundle/macos/omnix-gui.app

# Run the project locally
watch $RUST_BACKTRACE="1":
    # XXX: hot reload doesn't work with tailwind
    # dx serve --hot-reload
    dx serve --bin omnix-gui

alias w := watch

# Run tests
test:
    cargo test

# Run docs server (live reloading)
doc:
    cargo-doc-live

# Run CI locally
ci:
    nixci

clippy:
    cargo clippy --release --locked --all-targets --all-features -- --deny warnings

# Setup node_modules using Nix (invoked automatically by nix-shell)
node_modules NODE_PATH:
    rm -f ./e2e/node_modules
    ln -sf ${NODE_PATH} ./e2e/node_modules
