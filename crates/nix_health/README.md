# nix-health

nix-health[^old] is a program that checks the health of your Nix install. Additional health checks can be specified in the `flake.nix` of individual projects.

[^old]: nix-health originally began as a script <https://github.com/srid/nix-health> which is now deprecated.

## Checks performed

| Check | Configurable in `flake.nix`? |
| --- | --- |
| Flakes are enabled | - |
| Nix version is not too old | Yes |
| Nix runs natively (no rosetta) | Yes |
| Builds use multiple cores (`max-jobs`) | Yes |
| Nix Caches in use | Yes |
| $USER is in `trusted-users` | - |
| Direnv: installed and activated | Yes |
| Min RAM / Disk space | Yes |

Note that some checks are considered non-essential. For eg., the disk space check looks for 1TB+ disk space, but if the user is on a laptop with 256GB SSD, the check will report a warning instead of failing.

## Usage

nix-health is still in development. To run the development version,

```sh
nix run "github:juspay/nix-browser#nix-health"
```

To run nix-health along with health check configuration specified in a project flake, pass that flake as an argument. For eg., to run nix-health with additional checks from the nammayatri project, run:

```sh
# The argument can be any flake URL (including a local path)
nix run "github:juspay/nix-browser#nix-health" github:nammayatri/nammayatri
```

## Release Tasks

- [ ] Finalize behaviour and config schema
- [ ] Documentation, esp. on flake.nix overrides
    - Do we need a blog post?
- [ ] Release to crates.io (including `nix_rs`) and open nixpkgs PR
