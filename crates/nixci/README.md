# nixci

[![Crates.io](https://img.shields.io/crates/v/nixci.svg)](https://crates.io/crates/nixci)
[![Harmeless Code of Conduct](https://img.shields.io/badge/harmless-8A2BE2)](https://srid.ca/coc "This project follows the 'Harmlessness Code of Conduct'")

`nixci` builds all outputs in a flake, or optionally its [sub-flakes](https://github.com/hercules-ci/flake-parts/issues/119), which can in turn be used either in CI or locally. Using [devour-flake] it will automatically build the following outputs:

| Type                   | Output Key                                      |
| ---------------------- | ----------------------------------------------- |
| Standard flake outputs | `packages`, `apps`, `checks`, `devShells`       |
| NixOS                  | `nixosConfigurations.*`                         |
| nix-darwin             | `darwinConfigurations.*`                        |
| home-manager           | `legacyPackages.${system}.homeConfigurations.*` |

The [stdout] of `nixci` will be a list of store paths built.

[stdout]: https://en.wikipedia.org/wiki/Standard_streams#Standard_output_(stdout)

## Install

### From nixpkgs

`nixpkgs` contains [version 0.5.0](https://github.com/NixOS/nixpkgs/pull/320437) of nixci that you can install using `nix profile install nixpkgs#nixci` or run using `nix run nixpkgs#nixci`.

### From source

To install, run `nix profile install github:srid/nixci`. You can also use use `nix run github:srid/nixci` to run `nixci` directly off this repo without installing it.

## Usage

`nixci` accepts any valid [flake URL](https://nixos.asia/en/flake-url) or a Github PR URL.

```sh
# Run nixci on current directory flake
$ nixci # Or `nixci build` or `nixci build .`

# Run nixci on a local flake (default is $PWD)
$ nixci build ~/code/myproject

# Run nixci on a github repo
$ nixci build github:hercules-ci/hercules-ci-agent

# Run nixci on a github PR
$ nixci build https://github.com/srid/emanote/pull/451

# Run only the selected sub-flake
$ git clone https://github.com/srid/haskell-flake && cd haskell-flake
$ nixci build .#default.dev
```

### Using in Github Actions

#### Standard Runners

Add the following to your workflow file,

```yaml
      - uses: actions/checkout@v4
      - uses: DeterminateSystems/nix-installer-action@main
        with:
          extra-conf: |
            trusted-public-keys = cache.garnix.io:CTFPyKSLcx5RMJKfLo5EEPUObbA78b0YQ2DTCJXqr9g= cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=
            substituters = https://cache.garnix.io?priority=41 https://cache.nixos.org/
      - uses: yaxitech/nix-install-pkgs-action@v3
        with:
          packages: "github:srid/nixci"
      - run: nixci build
```

#### Self-hosted Runners with Job Matrix

> [!NOTE] 
> This currently requires an explicit nixci configuration in your flake, viz.: `nixci.default.root.dir = ".";`.

```yaml
jobs:
  configure:
    runs-on: self-hosted
    outputs:
      matrix: ${{ steps.set-matrix.outputs.matrix }}
    steps:
     - uses: actions/checkout@v4
     - id: set-matrix
       run: echo "matrix=$(nixci gh-matrix --systems=aarch64-linux,aarch64-darwin | jq -c .)" >> $GITHUB_OUTPUT
  nix:
    runs-on: self-hosted
    needs: configure
    strategy:
      matrix: ${{ fromJson(needs.configure.outputs.matrix) }}
      fail-fast: false
    steps:
      - uses: actions/checkout@v4
      - run: nixci build --systems "github:nix-systems/${{ matrix.system }}" ".#default.${{ matrix.subflake }}"
```

> [!TIP] 
> If your builds fail due to GitHub's rate limiting, consider passing `--extra-access-tokens` (see [an example PR](https://github.com/srid/nixos-flake/pull/55)). If you get rate limits when accessing `github:nix-systems`, use [this workaround](https://github.com/srid/nixci/issues/83#issuecomment-2225903229).

## Configuring

By default, `nixci` will build the top-level flake, but you can tell it to build sub-flakes by adding the following output to your top-level flake:

```nix
# myproject/flake.nix
{
  nixci.default = {
    dir1 = {
        dir = "dir1";
    };
    dir2 = {
        dir = "dir2";
        overrideInputs.myproject = ./.;
    };
  }
}
```

You can have more than one nixci configuration. For eg., `nixci .#foo` will run the configuration from `nixci.foo` flake output.

### Examples

Some real-world examples of how nixci is used with specific configurations:

- [services-flake](https://github.com/juspay/services-flake/blob/197fc1c4d07d09f4e01dd935450608c35393b102/flake.nix#L10-L24)
- [nixos-flake](https://github.com/srid/nixos-flake/blob/4af32875e7cc6df440c5f5cf93c67af41902768b/flake.nix#L29-L45)
- [haskell-flake](https://github.com/srid/haskell-flake/blob/d128c7329bfc73c3eeef90f6d215d0ccd7baf78c/flake.nix#L15-L67)
    - Here's [a blog post](https://twitter.com/sridca/status/1763528379188265314) that talks about how it is used in haskell-flake

## What it does

- Optionally, accept a flake config (`nixci.default`) to indicate sub-flakes to build, along with their input overrides
- Preliminary checks
    - Check that `flake.lock` is in sync
    - Check that the Nix version is not tool old (using [nix-health](https://github.com/juspay/nix-health))
- Use [devour-flake](https://github.com/srid/devour-flake) to build all flake outputs[^schema]
- Print the built store paths to stdout

[^schema]: Support for [flake-schemas](https://github.com/srid/devour-flake/pull/11) is planned

[devour-flake]: https://github.com/srid/devour-flake

## Discussion

For discussion of nixci, please post in our [Zulip](https://nixos.zulipchat.com/#narrow/stream/413950-nix).

## See also

- [github-nix-ci](https://github.com/juspay/github-nix-ci) - A simple NixOS & nix-darwin module for self-hosting GitHub runners (includes `nixci` by default)
- [jenkins-nix-ci](https://github.com/juspay/jenkins-nix-ci) - Jenkins NixOS module that supports `nixci` as a Groovy function
