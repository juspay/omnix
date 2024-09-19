# `om init`

> [!IMPORTANT]
> `om init` is available as a beta-feature.

The `om init` command provides a better [`nix flake init`](https://nix.dev/manual/nix/2.18/command-ref/new-cli/nix3-flake-init) experience. Specifically, it provides:

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

## Configuration spec {#spec}

Omnix templates can be defined by adding a `om.template` flake output. This should be an attrset of templates. The value should contain the keys `template` (referring to original flake template) as well as `params`, defined as follows:

There are two kinds of params. **String params** are defined as follows:

```nix
{
  name = "package-name";
  description = "Name of the Rust package";
  placeholder = "rust-nix-template";
}
```

Here, when prompting for this param, the user-provided value if any will replace the given `placeholder` text across all files in the template.

**Boolean params** are defined as follows:

```nix
{
  name = "vscode";
  description = "Include the VSCode settings folder (./.vscode)";
  paths = [ ".vscode" ];
  value = true;
}
```

Here, if the user enables this param, the path globs specified in `paths` will be retained in the template. Otherwise, the paths will be deleted. The `value` key provides a default value; which key is supported for string params as well.

Both parameter types are distinguished by the presence of the relevant keys (`placeholder` for string, `paths` for boolean).
