---
slug: /health
---

# Nix Health

[nix-health](https://github.com/juspay/nix-browser/tree/main/crates/nix_health)[^1] is a program that checks the health of your Nix install. Furthermore, individual projects can configure their own health checks in their `flake.nix`. For example, the nammayatri project checks that [the cachix cache is in use](https://github.com/nammayatri/nammayatri/blob/2201f618af69dc78070fefeb4f082420b1d226cc/flake.nix#L29-L31).

[^1]: nix-health originally began as a script <https://github.com/srid/nix-health> which is now deprecated.

![](https://github.com/juspay/nix-browser/assets/3998/abbbc54b-d888-42fb-a2a8-31d9ae142d6a)

## Checks performed

| Check | Configurable in `flake.nix`? |
| --- | --- |
| Flakes are enabled | - |
| Nix version is not too old | Yes |
| Nix runs natively (no rosetta)[^2] | Yes |
| Builds use multiple cores (`max-jobs`) | Yes |
| Nix Caches in use | Yes |
| $USER is in `trusted-users` | - |
| Direnv: installed and activated | Yes |
| Min RAM / Disk space | Yes |

[^2]: This check is only performed on macOS with Apple Silicon.

Note that some checks are considered non-essential. For eg., the disk space check looks for 1TB+ disk space, but if the user is on a laptop with 256GB SSD, the check will report a warning instead of failing. This can also be configured in per-project basis from `flake.nix` (see below).

## Usage

nix-health is still in development. To run the development version,

```bash
nix run "github:juspay/nix-browser#nix-health"
```

To run nix-health along with health check configuration specified in a project flake, pass that flake as an argument. For eg., to run nix-health with additional checks from the nammayatri project, run:

```bash
# The argument can be any flake URL (including a local path)
nix run "github:juspay/nix-browser#nix-health" github:nammayatri/nammayatri
```

## Configuring in `flake.nix` {#conf}

To add project specific health checks or configure health checks, add the following flake output:

```nix
{
  outputs = inputs: {
    nix-health.default = {
      # Add configuration here
      caches.required = [ "https://ourproject.cachix.org" ];
    };
  };
}
```

To see all available configuration options, run `nix-health --dump-schema`. This will dump the schema of the configuration in JSON format. Convert that to a Nix attrset to see what can be added under the `nix-health.default` attrset of your flake.

```sh-session
$ nix-health --dump-schema > schema.json
$ nix eval --impure --expr 'builtins.fromJSON (builtins.readFile ./schema.json)' | nix run nixpkgs#alejandra -- --quiet
{
  caches = {required = ["https://cache.nixos.org/"];};
  direnv = {
    enable = true;
    required = false;
  };
  flake-enabled = {};
  max-jobs = {};
  nix-version = {min-required = "2.13.0";};
  rosetta = {
    enable = true;
    required = true;
  };
  system = {
    enable = true;
    min_disk_space = "1024.0 GB";
    min_ram = null;
    required = false;
  };
  trusted-users = {};
}
```
