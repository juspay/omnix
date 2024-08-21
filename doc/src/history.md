# Release history

## Unreleased

### Enhancements

- `om health`
  - Display Nix installer used (supports DetSys installer)
- `om ci`
  - Support for remote builds over SSH (via `--on` option)
  - Avoid running `nix-store` command multiple times (#224)
  - Run `nix flake check` on all subflakes (#200)

### Fixes

- `om ci run`: The `--override-input` option mandated `flake/` prefix (nixci legacy) which is no longer necessary in this release.

### Backward-incompatible changes

- `om ci build` has been renamed to `om ci run`.

## 0.1.0 (2024-08-08) {#0.1.0}

Initial release of omnix.
