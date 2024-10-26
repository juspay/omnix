# Documentation targets
mod doc

default:
    @just --list

# Auto-format the source tree
fmt:
    treefmt

alias f := fmt

# Run omnix-cli locally
watch *ARGS:
    bacon --job run -- -- {{ ARGS }}

run *ARGS:
    cargo run -p omnix-cli {{ ARGS }}

alias w := watch

# Run CI locally
ci:
    nix run . ci

# Run CI locally in devShell (using cargo)
ci-cargo:
    cargo run -p omnix-cli -- ci run

[group('ci')]
clippy:
    cargo clippy --release --locked --all-targets --all-features -- --deny warnings

[group('ci')]
cargo-doc:
    cargo doc --release --all-features --workspace
