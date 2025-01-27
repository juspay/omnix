# `om health`

The `om health` command checks the health of your Nix install. Furthermore, individual projects can configure their own health checks in their `flake.nix`. For example, the nammayatri project checks that [the cachix cache is in use](https://github.com/nammayatri/nammayatri/blob/2201f618af69dc78070fefeb4f082420b1d226cc/flake.nix#L29-L31).


![](https://github.com/juspay/omnix/assets/3998/abbbc54b-d888-42fb-a2a8-31d9ae142d6a)

> [!NOTE]
> **History**: `om health` was formerly called [`nix-health`](https://github.com/juspay/nix-health).

## Checks performed

| Check                                  | Configurable in `flake.nix`? |
| -------------------------------------- | ---------------------------- |
| Flakes are enabled                     | -                            |
| Nix version is supported               | Yes                          |
| Nix runs natively (no rosetta)[^ros]   | Yes                          |
| Builds use multiple cores (`max-jobs`) | Yes                          |
| Nix Caches in use                      | Yes                          |
| $USER is in `trusted-users`            | -                            |
| Direnv: installed and activated        | Yes                          |
| Dotfiles are managed by Nix            | Yes                          |
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

## Per-project configuration {#conf}

To add project specific health checks or configure health checks, add the following to your [`om.yaml`](../config.md):

```yaml
health:
  default:
    caches:
      required:
        - "https://ourproject.cachix.org"
```

To see all available configuration options, run `om health --dump-schema`. This will dump the schema of the configuration in JSON format. Convert that to YAML to see what can be added under the `om.health.default` key of your [`om.yaml`](../config.md).

```sh-session
$ om health --dump-schema | nix run nixpkgs#yq-go -- -P
```

This will output:

```yaml
flake-enabled: {}
nix-version:
  supported: '>=2.16.0'
rosetta:
  enable: true
  required: true
max-jobs: {}
trusted-users: {}
caches:
  required:
    - https://cache.nixos.org/
direnv:
  enable: true
  required: false
shell:
  enable: true
  required: false
```

### Adding devShell check {#devshell}

You can automatically run `om health` as part of direnv invocation; see [`om develop`](develop.md) for details.
