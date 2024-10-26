{ inputs, ... }:
# Nix module for the Rust part of the project
#
# This uses Crane, via https://github.com/juspay/rust-flake
{
  imports = [
    inputs.rust-flake.flakeModules.default
    inputs.process-compose-flake.flakeModule
    inputs.cargo-doc-live.flakeModule
  ];
  perSystem = { config, self', pkgs, lib, ... }: {
    cargo-doc-live.crateName = "omnix-gui";

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
            # Select *only* the non-Rust files necessary to build omnix package.
            || lib.hasSuffix "flake-parts/nixpkgs.nix" path
            || lib.hasSuffix "flake-parts/rust.nix" path
            || lib.hasSuffix "tests/flake.nix" path
            || lib.hasSuffix "tests/flake.lock" path
            || lib.hasSuffix "failing/flake.nix" path
            || lib.hasSuffix "registry/flake.nix" path
            || lib.hasSuffix "registry/flake.lock" path
            || lib.hasSuffix "flake-schemas/flake.nix" path
            || lib.hasSuffix "flake-schemas/flake.lock" path
          ;
        };
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

    checks.omnix-source-has-flake = pkgs.runCommand
      "omnix-source-has-flake"
      { buildInputs = [ pkgs.lsd ]; } ''
      set -e
      cd ${self'.packages.omnix-cli.OMNIX_SOURCE.outPath}
      pwd
      lsd --tree
      test -f flake.nix
      test -f flake.lock
      touch $out
    '';
  };
}
