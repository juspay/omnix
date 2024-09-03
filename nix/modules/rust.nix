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
  perSystem = { inputs', config, self', pkgs, lib, system, ... }: {
    cargo-doc-live.crateName = "omnix-gui";

    rust-project = {
      # See /crates/*/crate.nix for the crate-specific Nix configuration
      crateNixFile = "crate.nix";
    };

    checks =
      let
        inherit (config.rust-project) crates;
      in
      {
        # Clippy checks
        # TODO: Remove after https://github.com/juspay/rust-flake/issues/23
        omnix-cli-clippy = crates."omnix-cli".crane.outputs.drv.clippy;
        omnix-init-clippy = crates."omnix-init".crane.outputs.drv.clippy;
        omnix-common-clippy = crates."omnix-common".crane.outputs.drv.clippy;
        nix_rs-clippy = crates."nix_rs".crane.outputs.drv.clippy;
        nixci-clippy = crates."nixci".crane.outputs.drv.clippy;
        nix_health-clippy = crates."nix_health".crane.outputs.drv.clippy;
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

        # TODO: remove this
        omnix-init = crates."omnix-init".crane.outputs.drv.crate;

        # Rust docs
        # TODO: Remove after https://github.com/juspay/rust-flake/issues/23
        omnix-cli-doc = crates."omnix-cli".crane.outputs.drv.doc;
        omnix-init-doc = crates."omnix-init".crane.outputs.drv.doc;
        omnix-common-doc = crates."omnix-common".crane.outputs.drv.doc;
        nix_rs-doc = crates."nix_rs".crane.outputs.drv.doc;
        nixci-doc = crates."nixci".crane.outputs.drv.doc;
        nix_health-doc = crates."nix_health".crane.outputs.drv.doc;

        /*
        gui = crates."omnix-gui".crane.outputs.drv.crate.overrideAttrs (oa: {
          # Copy over assets for the desktop app to access
          installPhase =
            (oa.installPhase or"") + ''
              cp -r ${inputs.self + /crates/omnix-gui/assets}/* $out/bin/
            '';
          postFixup =
            (oa.postFixup or"") + ''
              # HACK: The Linux desktop app is unable to locate the assets
              # directory, but it does look into the current directory.
              # So, `cd` to the directory containing assets (which is
              # `bin/`, per the installPhase above) before launching the
              # app.
              wrapProgram $out/bin/${oa.pname} \
                --chdir $out/bin
            '';
        });
        */

      };
  };
}
