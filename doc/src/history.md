# Release history

## 1.0.4 (UNRELEASED)

- `om ci`: Allow impure builds through `impure = true;` setting in `om.yaml` (#445)

## 1.0.3 (2025-03-17) {#1.0.3}

### Fixes

- `om ci`
  - Extra nix handling
      - Allow `--override-input` to work again (#439)
      - Support `--rebuild` by disallowing it in irrelevant subcommands (`eval`, `develop`, `run`, `flake {lock,check}`) (#441)
- `om init`
  - Handle symlinks *as is* (we expect relative symlink targets) without resolution (#443)

## 1.0.2 (2025-03-11) {#1.0.2}

### Fixes

- `om ci`
  - Prevent bad UTF-8 in build logs from crashing `om ci run` (#437)

## 1.0.1 (2025-03-10) {#1.0.1}

### Fixes

- `om init`
  - now copies over permissions as is (e.g.: respects executable bits on files) (#434)
  - applies replace in proper order so that directory rename doesn't skip content replace in its children  (#435)

### Chores

- Allow building on stable version of Rust (#427)
- Define ENVs in a single place and import them as default for all crates (#430)

## 1.0.0 (2025-02-17) {#1.0.0}

### Enhancements

- `om develop`: New command
- `om init`
  - Initial working version of `om init` command
- `om health`
  - Display Nix installer used (supports DetSys installer)
  - Display information in Markdown
  - Remove RAM/disk space checks, moving them to "information" section
  - Add shell check, to ensure its dotfiles are managed by Nix.
  - Add `--json` that returns the health check results as JSON
  - Switch from `nix-version.min-required` to more flexible `nix-version.supported`.
- `om ci`
  - Support for remote builds over SSH (via `--on` option)
  - Support for CI steps
    - Run `nix flake check` on all subflakes (#200)
    - Ability to add a custom CI step. For example, to run arbitrary commands.
  - Add `--accept-flake-config`
  - Add `--results=FILE` to store CI results as JSON in a file
  - Misc
    - Avoid running `nix-store` command multiple times (#224)
    - Locally cache `github:nix-systems` (to avoid Github API rate limit)

### Fixes

- `om ci run`: The `--override-input` option mandated `flake/` prefix (nixci legacy) which is no longer necessary in this release.
- `om health`: Use `whoami` to determine username which is more reliable than relying on `USER` environment variable

### Backward-incompatible changes

- `nix-health` and `nixci` flake output configurations are no longer supported.
- `om ci build` has been renamed to `om ci run`.

## 0.1.0 (2024-08-08) {#0.1.0}

Initial release of omnix.
