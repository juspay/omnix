# `om develop`

The `om develop` command should be used indirectly in direnv, via the `use omnix` directive in your `.envrc`.

## Getting started {#start}

1. Put this in your `.envrc` file:

    ```sh
    source_url \
      https://raw.githubusercontent.com/juspay/omnix/2c80a2b228a8c21434172d2b4f3b8d2ee873284f/omnixrc \
      'sha256-WywFYSB50xlznsc/tkFAF18uI2pMNoTJJAJJUPs837Y='
    watch_file om.yaml
    use omnix
    ```

2. You should also create an empty (or fleshed out) [`om.yaml`](../config.md) file in your project to avoid Nix evaluation:

    ```sh
    touch om.yaml
    ```


## What does it do? {#what}

`use omnix` wraps `use flake` (of [nix-direnv](https://nixos.asia/en/direnv)) providing additional capabilities:

- Run [`om health`](health.md) to check the health of the Nix environment.
  - Run `cachix use` automatically if the project uses cachix.
- Print a welcome text after spawning the Nix devshell.

The ideal goal here being that `cd`'ing to a project should do everything necessary to get you started immediately.

## Welcome text {#welcome}

The welcome text can be configured in your [`om.yaml`](../config.md) file. For example:

```yaml
develop:
  default:
    readme: |
      🍾 Welcome to the **omnix** project

      To run omnix,

      ```sh-session
      just watch <args>
      ```

      (Now, as you edit the Rust sources, the above will reload!)

      🍎🍎 Run 'just' to see more commands. See <https://nixos.asia/en/vscode> for IDE setup.
```
