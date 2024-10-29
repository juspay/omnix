# Develop

The `om develop` should be used indirectly in direnv, via the `use omnix` directive in your `.envrc`:

```sh
source_url https://raw.githubusercontent.com/juspay/omnix/4daebcb38082e0f933d6a25284948122ad3a507e/omnixrc 'sha256-6+bGgf1Dw9Ua/7aiFs7RyN8slHZeOsBCmNsIQ5nqHGM='

use omnix
```

`use omnix` wraps `use flake` (of nix-direnv) providing additional capabilities:

- Run [`om health`](health.md) to check the health of the Nix environment.
- Run `cachix use` automatically if the project uses cachix.
- Print a welcome text after spawning the Nix devshell.

The ideal goal here being that switching to a project should do everything necessary to get you started immediately.

## Welcome text {#welcome}

The welcome text can be configured in your om configuration:

```nix
{
  om.develop.default = {
    readme = ''
      Welcome to our **project**

      To get started, run the following:

      ```sh
      just run
      ```

      For more, read the README.md
    '';
  }
}
```
