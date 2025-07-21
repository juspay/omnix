# Documentation targets
mod doc

default:
    @just --list

# Run all pre-commit hooks on all files
pca:
    pre-commit run -a

# Run omnix-cli locally
watch *ARGS:
    bacon --job run -- -- {{ ARGS }}

run *ARGS:
    cargo run -p omnix-cli {{ ARGS }}

alias w := watch

# Run CI locally
[group('ci')]
ci:
    nix --accept-flake-config run . ci

# Run CI locally in devShell (using cargo)
[group('ci')]
ci-cargo:
    cargo run -p omnix-cli -- ci run

# Run CI locally in devShell (using cargo) on a simple flake with subflakes
[group('ci')]
ci-cargo-ext:
    cargo run -p omnix-cli -- ci run github:srid/nixos-unified

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

# Run impure tests that require network access (normally ignored in CI)
[group('ci-steps')]
cargo-test-impure:
    cargo test --release --all-features --workspace -- --ignored

# Run all tests including impure tests
[group('ci-steps')]
cargo-test-all:
    cargo test --release --all-features --workspace -- --include-ignored
