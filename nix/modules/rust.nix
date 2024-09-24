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
