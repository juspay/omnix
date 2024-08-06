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
      name = "omnix";
      meta.description = "Omnix development environment";
      inputsFrom = [
        config.treefmt.build.devShell
        config.nix-health.outputs.devShell
        self'.devShells.rust
      ];
      OM_INIT_REGISTRY = inputs.self + /crates/flakreate/registry;
      NIX_FLAKE_SCHEMAS_BIN = lib.getExe (if pkgs.stdenv.isLinux then inputs'.nix.packages.nix-static else inputs'.nix.packages.default);
      DEFAULT_FLAKE_SCHEMAS = inputs.flake-schemas;
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
