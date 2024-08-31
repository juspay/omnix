# Release history

## Unreleased

### Enhancements

- `om health`
  - Display Nix installer used (supports DetSys installer)
- `om ci`
  - Support for remote builds over SSH (via `--on` option)
  - Avoid running `nix-store` command multiple times (#224)
  - Support for CI steps
    - Run `nix flake check` on all subflakes (#200)
    - Ability to add a custom CI step. For example, to run arbitrary commands.

### Fixes

- `om ci run`: The `--override-input` option mandated `flake/` prefix (nixci legacy) which is no longer necessary in this release.

### Backward-incompatible changes

- `om ci build` has been renamed to `om ci run`.

## 0.1.0 (2024-08-08) {#0.1.0}

Initial release of omnix.
