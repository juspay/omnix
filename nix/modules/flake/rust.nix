{ inputs, ... }:
# Nix module for the Rust part of the project
#
# This uses Crane, via https://github.com/juspay/rust-flake
{
  imports = [
    inputs.rust-flake.flakeModules.default
  ];
  perSystem = { config, self', pkgs, lib, ... }: {
    rust-project = {
      # See /crates/*/crate.nix for the crate-specific Nix configuration
      crateNixFile = "crate.nix";

      # To avoid unnecessary rebuilds, start from cleaned source, and then add the Nix files necessary to `nix run` it. Finally, add any files required by the Rust build.
      src =
        let
          # Like crane's filterCargoSources, but doesn't blindly include all TOML files!
          filterCargoSources = path: type:
            config.rust-project.crane-lib.filterCargoSources path type
            && !(lib.hasSuffix ".toml" path && !lib.hasSuffix "Cargo.toml" path);
        in
        lib.cleanSourceWith {
          src = inputs.self;
          filter = path: type:
            filterCargoSources path type
            || lib.hasSuffix "registry.json" path
            || lib.hasSuffix "crate.nix" path
            || "${inputs.self}/flake.nix" == path
            || "${inputs.self}/flake.lock" == path
            || "${inputs.self}/rust-toolchain.toml" == path
            # Select *only* the non-Rust files necessary to build omnix package.
            || lib.hasSuffix "envs/default.nix" path
            || lib.hasSuffix "flake/nixpkgs.nix" path
            || lib.hasSuffix "flake/rust.nix" path
            || lib.hasSuffix "tests/flake.nix" path
            || lib.hasSuffix "tests/flake.lock" path
            || lib.hasSuffix "failing/flake.nix" path
            || lib.hasSuffix "registry/flake.nix" path
            || lib.hasSuffix "registry/flake.lock" path
            || lib.hasSuffix "flake-schemas/flake.nix" path
            || lib.hasSuffix "flake-schemas/flake.lock" path
            || lib.hasSuffix "addstringcontext/flake.nix" path
            || lib.hasSuffix "addstringcontext/flake.lock" path
            || lib.hasSuffix "metadata/flake.nix" path
            || lib.hasSuffix "metadata/flake.lock" path
          ;
        };
      defaultCraneArgs = import "${inputs.self}/nix/envs" { inherit (config.rust-project) src; inherit (pkgs) cachix fetchFromGitHub lib; };
    };

    packages =
      let
        inherit (config.rust-project) crates;
      in
      rec {
        default = omnix-cli;
        omnix-cli = crates."omnix-cli".crane.outputs.drv.crate.overrideAttrs (oa: {
          nativeBuildInputs = (oa.nativeBuildInputs or [ ]) ++ [ pkgs.installShellFiles ];
          postInstall = ''
            installShellCompletion --cmd om \
              --bash <($out/bin/om completion bash) \
              --zsh <($out/bin/om completion zsh) \
              --fish <($out/bin/om completion fish)
          '';
        });
      };

    apps.omnix-source-is-buildable.program = pkgs.writeShellApplication {
      name = "omnix-source-is-buildable";
      runtimeInputs = [
        pkgs.jq
      ];
      text = ''
        set -e
        cd ${self'.packages.omnix-cli.OMNIX_SOURCE}
        # Make sure the drv evaluates (to test that no files are accidentally excluded)
        nix --accept-flake-config --extra-experimental-features "flakes nix-command" \
          derivation show "." | jq -r '.[].outputs.out.path'
      '';
    };
  };
}
