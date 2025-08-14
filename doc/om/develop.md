# `om develop`

The `om develop` command should be used indirectly in direnv, via the `use omnix` directive in your `.envrc`.

## Getting started {#start}

1. Put this in your `.envrc` file:

    ```sh
    source_url \
      https://omnix.page/om/develop/omnixrc/v1 \
      'sha256-FBAVRYkaexKeFKQGUxaPHqhBnqA7km7++O77dKiyD0I='
    watch_file om.yaml
    use omnix
    ```

2. You should also create a [`om.yaml`](../config.md) file in your project to avoid Nix evaluation:

    ```sh
    touch om.yaml # Can be empty
    ```

3. Optionally, add a welcome text (see below) to that `om.yaml`. See [this commit](https://github.com/srid/haskell-template/commit/128105dbeac47c515065ba377f4b1f976ec4f696) for a full example.

## What does it do? {#what}

`use omnix` wraps `use flake` (of [nix-direnv](https://nixos.asia/en/direnv)) providing additional capabilities:

- Run [`om health`](health.md) to check the health of the Nix environment.
  - **Automatically setup missing caches** if they're configured in the project:
    - Run `cachix use <name>` for Cachix caches (URLs like `https://name.cachix.org`)
    - Run `attic login` and `attic use` for Attic caches (URLs like `attic+server+https://...`)
    - Report other missing caches that must be manually configured
- Print a welcome text after spawning the Nix devshell.

The ideal goal here being that `cd`'ing to a project should do everything necessary to get you started immediately.

## Automatic Cache Setup {#caches}

Unlike `om health` which only *checks* for missing caches, `om develop` will automatically *setup* them:

### Cachix Caches
If your project's `om.yaml` includes Cachix URLs like `https://yourproject.cachix.org`, `om develop` will automatically run:
```bash
cachix use yourproject
```

### Attic Caches  
If your project includes Attic URLs like `attic+server+https://cache.example.com/name`, `om develop` will automatically run:
```bash
attic login server https://cache.example.com/name $ATTIC_LOGIN_TOKEN
attic use server:name
```

**Important**: For Attic caches, set the `ATTIC_LOGIN_TOKEN` environment variable (can be empty for public caches).

### Manual Configuration Required
Standard HTTPS cache URLs that don't match the above patterns cannot be automatically setup and must be manually added to your Nix configuration.

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

## Revision History for `omnixrc` {#omnixrc-history}

### v1

- Initial release using pinned nixpkgs for omnix `1.3.0`.
