default:
    @just --list

# Auto-format the source tree
fmt:
    treefmt
    # TODO: Integrate this to treefmt: https://github.com/numtide/treefmt-nix/issues/106
    leptosfmt src

# Run the project locally
watch *ARGS:
    cargo leptos watch

# Run tests (backend & frontend)
test:
    cargo watch -- cargo leptos test

doc:
    cargo-docs-server
