
## [1.0.0](https://github.com/srid/nixci/compare/0.5.0...1.0.0) (2024-07-23)

### Features

* add shell completions (#87)
([1b2caf3](https://github.com/srid/nixci/commit/1b2caf369c739382e2f1c22bfb32096f65addfba)),
closes [#87](https://github.com/srid/nixci/issues/87)
* **build:** Check for minimum nix version before running nixci (#75)
([ac5a011](https://github.com/srid/nixci/commit/ac5a011c76e9537426e0265b20e46f8efea44d40)),
closes [#75](https://github.com/srid/nixci/issues/75)
* **cli:** Allow `--override-input` to refer to flake name without `flake/`
prefix of devour_flake (#74)
([c17f42f](https://github.com/srid/nixci/commit/c17f42f3480b4b265bac0d94e7169ca01201fb9d)),
closes [#74](https://github.com/srid/nixci/issues/74)

### Fixes

* `--print-all-dependencies` should ignore `unknown-deriver` (#76)
([d26bab1](https://github.com/srid/nixci/commit/d26bab116f19ac248a7073de9de3ae8a3ac0271f)),
closes [#76](https://github.com/srid/nixci/issues/76)
* `--print-all-dependencies` should handle `unknown-deriver` (#70)
([16815b6](https://github.com/srid/nixci/commit/16815b6c9e476defd993368d0957335f86f9c055)),
closes [#70](https://github.com/srid/nixci/issues/70)

## [0.5.0](https://github.com/srid/nixci/compare/0.4.0...0.5.0) (2024-06-15)

### Features

* Avoid fetching for known `--system` combinations
([6164d6c](https://github.com/srid/nixci/commit/6164d6c6d37ccab02ddc4943962fd7c21828054c))
* **api:** Pass `NixCmd` explicitly around
([6a672e2](https://github.com/srid/nixci/commit/6a672e28811f716a8cff5108dc720269d897d246))
* Accept global options to pass to Nix
([cca8b98](https://github.com/srid/nixci/commit/cca8b988e24d5d4e7d76e6d2398a0f2e0b686abf))
* **cli:** add `--print-all-depedencies` to `nixci build` subcommand (#60)
([4109ce9](https://github.com/srid/nixci/commit/4109ce9982ad2f54e769c302ab044f16f8bd865c)),
closes [#60](https://github.com/srid/nixci/issues/60)

## 0.4.0 (Apr 19, 2024)

- New features
    - Add new config `nixci.*.*.systems` acting as a whitelist of systems to build that subflake.
    - Add `nixci build --systems` option to build on an arbitrary systems (\#39)
    - Allow selecting sub-flake to build, e.g.: `nixci .#default.myflake`  (\#45)
    - Add subcommand to generate Github Actions matrix (\#50)
        - Consequently, you must run `nixci build` instead of `nixci` now.
    - Pass `--extra-experimental-features` only when necessary. Simplifies logging. (#46)
- Fixes
    - Fix regression in Nix 2.19+ (`devour-flake produced an outpath with no outputs`) (\#35)
    - Evaluate OS configurations for current system only (\#38)
    - Fail correctly if nixci is passed a missing flake attribute (\#44)

## 0.2.0 (Sep 14, 2023)

- Breaking changes
    - Change flake schema: evaluates `nixci.default` instead of `nixci`; this allows more than one configuration (#20)
- Pass the rest of CLI arguments after `--` as-is to `nix build`
    - Consequently, remove `--rebuild`, `--no-refresh` and `--system` options, because these can be specified using the new CLI spec.
- Bug fixes
    - Fix nixci breaking if branch name of a PR has `#` (#17)
- Misc changes
    - Iterate configs in a deterministic order
    - stdout outputs are uniquely printed, in sorted order
    - stderr output is now logged using the `tracing` crate.
    - Pass `--extra-experimental-features` to enable flakes
    - `nixci` can now be used as a Rust library
    - `nixci` no longer depends on `devour-flake` the *executable package*, only on the flake.

## 0.1.3

- Pass `-j auto` to nix builds.

## 0.1.2

Initial release
