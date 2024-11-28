# Develop

The `om develop` should be used indirectly in direnv, via the `use omnix` directive in your `.envrc`:

```sh
source_url https://raw.githubusercontent.com/juspay/omnix/75ed48923835963e2f18baba08f54a8adc307ba2/omnixrc "sha256-8C2Jb5bHx/0cvm1+9gOlBEdWzbikCWT5UsJWewUAFt4="

use omnix
```

`use omnix` wraps `use flake` (of nix-direnv) providing additional capabilities:

- Run [`om health`](health.md) to check the health of the Nix environment.
- Run `cachix use` automatically if the project uses cachix.
- Print a welcome text after spawning the Nix devshell.

The ideal goal here being that switching to a project should do everything necessary to get you started immediately.

## `om.yaml`

You should also create a `om.yaml` (empty file if there's no configuration) so your flake is not evaluated during direnv.

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
