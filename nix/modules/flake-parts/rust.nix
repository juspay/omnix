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
  perSystem = { config, self', pkgs, ... }: {
    cargo-doc-live.crateName = "omnix-gui";

    rust-project = {
      # See /crates/*/crate.nix for the crate-specific Nix configuration
      crateNixFile = "crate.nix";
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

    # Make sure that `OMNIX_SOURCE` is buildable.
    # We must check this in CI, because it is built only during runtime otherwise (as part of `om ci run --on`).
    apps.build-omnix-source = {
      meta.description = "Build OMNIX_SOURCE";
      program = pkgs.writeShellApplication {
        name = "build-omnix-source";
        runtimeInputs = [ pkgs.nix ];
        text = ''
          set -x
          nix build -L --no-link --print-out-paths ${self'.packages.omnix-cli.OMNIX_SOURCE.outPath}
        '';
      };
    };
  };
}
