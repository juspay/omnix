# `om health`

The `om health` command checks the health of your Nix install. Furthermore, individual projects can configure their own health checks in their `flake.nix`. For example, the nammayatri project checks that [the cachix cache is in use](https://github.com/nammayatri/nammayatri/blob/2201f618af69dc78070fefeb4f082420b1d226cc/flake.nix#L29-L31).


![](https://github.com/juspay/omnix/assets/3998/abbbc54b-d888-42fb-a2a8-31d9ae142d6a)

> [!NOTE]
> **History**: `om health` was formerly called [`nix-health`](https://github.com/juspay/nix-health).

## Checks performed

| Check                                  | Configurable in `flake.nix`? |
| -------------------------------------- | ---------------------------- |
| Flakes are enabled                     | -                            |
| Nix version is not too old             | Yes                          |
| Nix runs natively (no rosetta)[^ros]   | Yes                          |
| Builds use multiple cores (`max-jobs`) | Yes                          |
| Nix Caches in use                      | Yes                          |
| $USER is in `trusted-users`            | -                            |
| Direnv: installed and activated        | Yes                          |
| Min RAM / Disk space                   | Yes                          |

[^ros]: This check is only performed on macOS with Apple Silicon.

Note that some checks are considered non-essential. For eg., the disk space check looks for 1TB+ disk space, but if the user is on a laptop with 256GB SSD, the check will report a warning instead of failing. This can also be configured in per-project basis from `flake.nix` (see below).

## Usage

```bash
om health
```

To run use the health check configuration specified in a project flake, pass that flake as an argument. For eg., to run halth checks defined from the nammayatri project, run:

```bash
# The argument can be any flake URL (including a local path)
om health github:nammayatri/nammayatri
```

## Configuring in `flake.nix` {#conf}

To add project specific health checks or configure health checks, add the following flake output:

```nix
{
  outputs = inputs: {
    om.health.default = {
      # Add configuration here
      caches.required = [ "https://ourproject.cachix.org" ];
    };
  };
}
```

To see all available configuration options, run `om health --dump-schema`. This will dump the schema of the configuration in JSON format. Convert that to a Nix attrset to see what can be added under the `om.health.default` attrset of your flake.

```sh-session
$ om health --dump-schema > schema.json
$ nix eval --impure --expr 'builtins.fromJSON (builtins.readFile ./schema.json)' \
  | nix run nixpkgs#alejandra -- --quiet
```

This will output:

```nix
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

### Adding devShell check {#devshell}

> [!WARNING]
> This section needs to be finalized for omnix. See [here](https://github.com/srid/haskell-template/pull/139/files) for the up-to-date proof of concept.

You can automatically run `om health` whenever your Nix dev shell starts. To do this, import the flake module in your flake and use it in your devShell:

```nix
{
  inputs = {
    omnix-flake.url = "github:juspay/omnix?dir=nix/om";
  };
  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.omnix-flake.flakeModules.default
      ];
      perSystem = { config, pkgs, ... }: {
        devShells.default = pkgs.mkShell {
          inputsFrom = [
            config.om.health.outputs.devShell
          ]
        };
      };
    };
}
```

Now suppose you have Nix 2.18 installed, but your project requires 2.19 or above due to the following config in its `flake.nix`:

```nix
flake.om.health.default = {
  nix-version.min-required = "2.19.0";
};
```

you can expect the devShell to print a giant message like this:

<img width="501" alt="image" src="https://github.com/juspay/nix-health/assets/3998/9f3b3141-611f-484f-b897-3e375c02dff5">

Note that you will still be dropped into the Nix dev shell (there's no way to abrupt the launching of a dev Shell).
