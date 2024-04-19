{
  description = "WIP: nix-browser";
  nixConfig = {
    # https://garnix.io/docs/caching
    extra-substituters = "https://cache.garnix.io";
    extra-trusted-public-keys = "cache.garnix.io:CTFPyKSLcx5RMJKfLo5EEPUObbA78b0YQ2DTCJXqr9g=";
  };
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";

    rust-flake.url = "github:juspay/rust-flake";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
    process-compose-flake.url = "github:Platonic-Systems/process-compose-flake";
    cargo-doc-live.url = "github:srid/cargo-doc-live";
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
        ./rust.nix
      ];

      flake = {
        nix-health.default = {
          nix-version.min-required = "2.16.0";
          caches.required = [ "https://cache.garnix.io" ];
          direnv.required = true;
          system = {
            # required = true;
            min_ram = "16G";
            # min_disk_space = "2T";
          };
        };
      };

      perSystem = { config, self', pkgs, lib, system, ... }: {
        # Add your auto-formatters here.
        # cf. https://numtide.github.io/treefmt/
        treefmt.config = {
          projectRootFile = "flake.nix";
          programs = {
            nixpkgs-fmt.enable = true;
            rustfmt.enable = true;
          };
        };

        devShells.default = pkgs.mkShell {
          name = "nix-browser";
          inputsFrom = [
            config.treefmt.build.devShell
            self'.devShells.rust
          ];
          packages = with pkgs; [
            just
            # For when we start using Tauri
            cargo-tauri
            trunk
          ];
          shellHook = ''
            echo
            echo "🍎🍎 Run 'just <recipe>' to get started"
            just
          '';
        };
      };
    };
}
