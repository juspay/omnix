# `om init`

The `om init` command provides a better [`nix flake init`](https://nix.dev/manual/nix/2.18/command-ref/new-cli/nix3-flake-init) experience. Specifically, it provides:

1. a registry of flake templates that you can choose from
2. support for template paramters that can be filled in by the user

To get started, run:

```sh
om init -o ~/myproject
```

This will prompt you to choose a template from the builtin registry (see below section), and then initialize it in the `myproject` directory.

## Builtin registry {#registry}

The builtin registry (stored in [a JSON file][json]) contains the following templates:

- [Haskell project template](https://github.com/srid/haskell-template)
- [Rust project template](https://github.com/srid/rust-nix-template)
- [NixOS/ nix-darwin/ home-manager template](https://github.com/juspay/nixos-unified-template)

[json]: https://github.com/juspay/omnix/blob/main/crates/omnix-init/registry/registry.json

## Initializing your own project templates {#custom}

If your flake provides a `om.templates` output (see below section), then `om init` will recognize it. For example:

```sh
om init -o ~/myproject github:srid/haskell-flake
```

Because haskell-flake has [a `om.templates` output](https://github.com/srid/haskell-flake/blob/31d7f050935f5a543212b7624d245f918ab14275/flake.nix#L16-L26), `om init` will prompt you to fill in the parameters defined in the template and initialize it.

You can also explicitly specify the template to choose from the flake:

```sh
om init -o ~/myproject github:srid/haskell-flake#haskell-flake
```

If there are multiple templates in the flake (as is the case with the builtin registry), omnix will the prompt the user to choose from them.

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

## Testing templates {#test}

The configuration can also include a `tests` key that defines a list of tests to run on the template. Each test is an attrset with `params` and `asserts` keys that indicates the parameter values to test along with the path existance assertions. [For example](https://github.com/juspay/nixos-unified-template/blob/3c4428ac94a4582a33e6fb3fe18df27bbc1e9eb7/modules/flake-parts/template.nix#L139-L157):

```nix
{
  tests = {
    default = {
      # systems = [ ]; # Optional whitelist of systems to limit this test to
      params = {
        username = "john";
        git-email = "john@ex.com";
        git-name = "John Doe";
        neovim = true;
      };
      asserts = {
        # Path assertion tests under template output
        source = {
          # true means the path must exist; false means it must not exist
          "modules/home/neovim/default.nix" = true;
          ".github/workflows" = false;
        };
        # Path assertion tests under the output of a Nix package
        packages."homeConfigurations.john.activationPackage" = {
          "home-path/bin/nvim" = true;
        };
      };
    };
  };
}
```
