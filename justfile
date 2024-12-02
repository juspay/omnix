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
[group('ci')]
ci:
    nix run . ci

# Run CI locally in devShell (using cargo)
[group('ci')]
ci-cargo:
    cargo run -p omnix-cli -- ci run

# Do clippy checks for all crates
[group('ci-steps')]
clippy:
    cargo clippy --release --locked --all-targets --all-features --workspace -- --deny warnings

# Build cargo doc for all crates
[group('ci-steps')]
cargo-doc:
    cargo doc --release --all-features --workspace

# Run cargo test for all crates
[group('ci-steps')]
cargo-test:
    cargo test --release --all-features --workspace
