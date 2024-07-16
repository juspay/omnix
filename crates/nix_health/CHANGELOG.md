# Changelog

## [0.4.0](https://github.com/juspay/nix-health/compare/0.3.0...0.4.0) (2024-07-10)

### Features

* **lib:** Expose `print_returning_exit_code`  (#25)
([b9c70a9](https://github.com/juspay/nix-health/commit/b9c70a9506823bdcc1d54c14b7c56d299b3a5c6a)),
closes [#25](https://github.com/juspay/nix-health/issues/25)
* build linux static executable
([78b95e8](https://github.com/juspay/nix-health/commit/78b95e8528282ef3f88b2ed29c0f5fc0cebbaa07))
* Add flake-module to run nix-health in devShell shellHook
([2f8d8dc](https://github.com/juspay/nix-health/commit/2f8d8dc30121923192c78a8f5152c5c89fdf1809))

### Fixes

* build failure on intel mac
([91e9bcf](https://github.com/juspay/nix-health/commit/91e9bcfd60d672074951d534d7b51f609dda1e94))

## 0.3.0 (2024-07-10)

### Fixes

* **nix-health:** use `direnv status --json` & create `direnv` crate (#123)
([f7762d7](https://github.com/juspay/nix-health/commit/f7762d7fec28f3091289fb03b3ad171cfb923f87)),
closes [#123](https://github.com/juspay/nix-health/issues/123)
