{ inputs, ... }:

let
  root = ../..;
in
{
  imports = [
    (root + /crates/nix_health/module/flake-module.nix)
  ];

  perSystem = { config, self', inputs', pkgs, lib, ... }: {
    devShells.default = pkgs.mkShell {
      name = "omnix-devshell";
      meta.description = "Omnix development environment";
      inputsFrom = [
        config.treefmt.build.devShell
        config.nix-health.outputs.devShell
        self'.devShells.rust
      ];
      inherit (config.rust-project.crates."omnix-cli".crane.args)
        NIX_FLAKE_SCHEMAS_BIN
        DEFAULT_FLAKE_SCHEMAS
        OMNIX_SOURCE
        ;
      # TODO: move up
      inherit (config.rust-project.crates."omnix-init".crane.args)
        OM_INIT_REGISTRY;

      packages = with pkgs; [
        just
        cargo-watch
        cargo-expand
        cargo-nextest
        config.process-compose.cargo-doc-live.outputs.package
        # For when we start using Tauri
        cargo-tauri
        trunk
        mdbook
        mdbook-alerts
      ];
      shellHook =
        ''
          # For nixci
          export DEVOUR_FLAKE=${inputs.devour-flake}
        '' +
        ''
          echo
          echo "🍎🍎 Run 'just <recipe>' to get started"
          just
        '';
    };
  };
}
