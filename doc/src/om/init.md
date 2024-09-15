# `om init`

> [!IMPORTANT]
> `om init` is available as a beta-feature.

The `om init` command provides a better `nix flake init` experience. Specifically, it provides:

1. a registry of flake templates that you can choose from
2. support for template paramters that can be filled in by the user

## Available templates {#list}

| Description | Command |
|-------------|---------|
| [Haskell project template](https://github.com/srid/haskell-template) | `om init haskell-template` |
| [Rust project template](https://github.com/srid/rust-nix-template) | `om init rust-nix-template` |
| [home-manager template](https://github.com/juspay/nix-dev-home) | `om init nix-dev-home` |

## Adding your own project templates {#custom}

In future, you would be able to directly initialize a project from a git repository, viz.: `om init <url>`. This is explicitly not yet supported right now, because:

> [!NOTE]
> The specification for template paramters are yet to be finalized. Until, then the relevant parameter configuration is tied to the registry in omnix repo. See [`crates/omnix-init/registry`](https://github.com/juspay/omnix/tree/main/crates/omnix-init/registry).
