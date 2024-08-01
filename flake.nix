{
  nixConfig = {
    extra-substituters = "https://om.cachix.org";
    extra-trusted-public-keys = "om.cachix.org-1:ifal/RLZJKN4sbpScyPGqJ2+appCslzu7ZZF/C01f2Q=";
  };
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";

    # TODO: Use upstream after https://github.com/NixOS/nix/pull/8892
    # Note: This version of nix is only used to run `nix flake show` in omnix-cli
    # Also note: Using shivaraj-bh fork of nix which fixes x86_64-darwin on top of github:DeterminateSystems/nix-src/flake-schemas
    nix.url = "github:shivaraj-bh/nix/flake-schemas";

    rust-flake.url = "github:juspay/rust-flake";
    rust-flake.inputs.nixpkgs.follows = "nixpkgs";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
    process-compose-flake.url = "github:Platonic-Systems/process-compose-flake";
    cargo-doc-live.url = "github:srid/cargo-doc-live";

    devour-flake.url = "github:srid/devour-flake";
    devour-flake.flake = false;
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;

      imports = [
        inputs.treefmt-nix.flakeModule
        inputs.process-compose-flake.flakeModule
        inputs.cargo-doc-live.flakeModule
        inputs.rust-flake.flakeModules.default
        inputs.rust-flake.flakeModules.nixpkgs
        ./nix/rust.nix
        ./nix/closure-size.nix
        ./nix/cache-pins.nix
        ./crates/nix_health/module/flake-module.nix
      ];

      # omnix configuration
      flake.om = {
        health.default = {
          nix-version.min-required = "2.16.0";
          caches.required = [ "https://om.cachix.org" ];
          direnv.required = true;
          system = {
            # required = true;
            min_ram = "16G";
            # min_disk_space = "2T";
          };
        };
      };

      perSystem = { inputs', config, self', pkgs, lib, system, ... }: {
        # Add your auto-formatters here.
        # cf. https://nixos.asia/en/treefmt
        treefmt.config = {
          projectRootFile = "flake.nix";
          programs = {
            nixpkgs-fmt.enable = true;
            rustfmt.enable = true;
          };
        };

        # https://om.cachix.org will ensure that these paths are always
        # available. The rest may be be GC'ed.
        cache-pins.pathsToCache = {
          cli = self'.packages.default;
          nix-health = self'.packages.nix-health;
        };

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
          DEFAULT_FLAKE_SCHEMAS = "github:DeterminateSystems/flake-schemas";
          packages = with pkgs; [
            just
            cargo-watch
            cargo-expand
            cargo-nextest
            config.process-compose.cargo-doc-live.outputs.package
            # For when we start using Tauri
            cargo-tauri
            trunk
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
    };
}
