# Changelog

## Unreleased

- **`flake::url`**:
  - Remove `qualified_attr` module
- **`eval::nix_eval`**
  - Display evaluation progress
  - Decrease logging verbosity
- **`flake::schema`**
  - Don't hardcode flake schema types
- **`config`**
  - Don't enable flakes during `NixConfig::get`
- Support Nix 2.20
- **`flake::url`**
  - Add `without_attr`, `get_attr`
  - Simplify the return type of `RootQualifiedAttr::eval_flake`
  - Add `AsRef`, `Deref`, `From<&Path>` instances for `FlakeUrl`
  - `Path` instances for `FlakeUrl` no longer use the `path:` prefix (to avoid store copying)
  - **`attr`**:
    - Add `FlakeAttr::new` and `FlakeAttr::none` constructors
  - `qualified_attr` - vastly simplify module
- `flake::functions`:
  - Add new module
- **`flake::command`**:
  - Add module, for `nix run`, `nix build` and `nix develop`
- **`store`**:
  - Add module (upstreamed from nixci)
  - Add `StoreURI`
  - Avoid running `nix-store` multiple times.
- **`copy`**:
  - Takes `NixCopyOptions` now.
- **`env`**:
  - use `whoami` crate to find the current user instead of depending on environment variable `USER`
  - `NixEnv::detect`'s logging uses DEBUG level now (formerly INFO)
  - Add Nix installer to `NixEnv`
- **`command`
  - `run_with_args` is now `run_with`, and takes a function that mutates the `Command` at will.
  - Add `trace_cmd_with`
- **`version`**:
  - Add `NixVersion::get`
- **`system_list`**: New module
- **version_spec**: New `NixVersion` spec module

## 1.0.0

- **DeterminateSystems/flake-schemas**
  - Allow overriding the `nix` CLI command.
  - Switch to flake schema given by <https://github.com/DeterminateSystems/flake-schemas>
- **`flake::schema::FlakeSchema`**
  - Add `nixos_configurations`
- **`flake::url`**
  - `Flake::from_nix` explicitly takes `NixConfig` as argument, rather than implicitly running nix to get it.
  - Remove string convertion implementations; use `std::parse` instead, and handle errors explicitly.
  - Split attr code to its own module, `flake::url::attr`
  - Introduce `flake::url::qualified_attr` module
- **`eval`**
  - `nix_eval_attr_json`
    - No longer takes `default_if_missing`; instead (always) returns `None` if attribute is missing.
    - Rename to `nix_eval_maybe` (as there is no non-JSON variant)
- **`env::NixEnv`**
  - Clarify error message when `$USER` is not set
- **``command`**
  - Add `NixCmd::get()` to return flakes-enabled global command
  - `NixCmd::default()` returns the bare command (no experimental features enabled)
- ``config``
  - Add `NixConfig::get()` to get the once-created static value of `NixConfig`
- `info`
  - Add `NixInfo::get()` to get the once-created static value of `NixInfo`
  - Rename `NixInfo::from_nix()` to `NixInfo::new()`; the latter explicitly takes `NixConfig`

## [0.5.0](https://github.com/juspay/nix-rs/compare/0.4.0...0.5.0) (2024-06-05)

### Features

- Improve `with_flakes` to transform existing `NixCmd`
([f936e54](https://github.com/juspay/nix-rs/commit/f936e5401d1bc9b82084cf7b49402a5ee1a3b733))
- Add support for clap deriving
([f61bd2c](https://github.com/juspay/nix-rs/commit/f61bd2c740a23a10bbb89dfbd3b77fd4b2a49bac))
- Add `NixCmd::extra_access_tokens`
([a287ab2](https://github.com/juspay/nix-rs/commit/a287ab2ad2d21db6ac89e4ce94c55446a02af241))

## [0.4.0](https://github.com/juspay/nix-rs/compare/0.3.3...0.4.0) (2024-06-03)

### Features

- add `NixCmd::run_with_args`
([47f3170](https://github.com/juspay/nix-rs/commit/47f3170d57b72089eb977620217613571c52f456))
- add `FlakeUrl::with_attr`
([1ff343d](https://github.com/juspay/nix-rs/commit/1ff343d25f1a633c3caf2d6f723bbd1c9e352cbc))

### [0.3.3](https://github.com/juspay/nix-rs/compare/0.3.2...0.3.3) (2024-04-17)

#### Features

- **eval:** nix_eval_attr_json explicitly takes NixCmd
([cccdb43](https://github.com/juspay/nix-rs/commit/cccdb437f4f2b31d32778e9cf3de2ab1a61d9331))
- **command:** Add `with_flakes` returning smarter nix CLI with flakes enabled
([f7f217a](https://github.com/juspay/nix-rs/commit/f7f217a12acefc3992b5ff8ba59d861f5cc2abcb))

### 0.3.2 (2024-04-04)
