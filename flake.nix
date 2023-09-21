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

    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
    process-compose-flake.url = "github:Platonic-Systems/process-compose-flake";
    cargo-doc-live.url = "github:srid/cargo-doc-live";

    leptos-fullstack.url = "github:srid/leptos-fullstack";
    leptos-fullstack.flake = false;
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;
      imports = [
        inputs.treefmt-nix.flakeModule
        inputs.process-compose-flake.flakeModule
        inputs.cargo-doc-live.flakeModule
        (inputs.leptos-fullstack + /nix/flake-module.nix)
        ./e2e/flake-module.nix
      ];
      flake = {
        nix-health.default = {
          nix-version.min-required = "2.16.0";
          caches.required = [ "https://cache.garnix.io" ];
          direnv.required = true;
          system = {
            min_ram = "16G";
            # min_disk_space = "2T";
          };
        };
      };
      perSystem = { config, self', pkgs, lib, system, ... }: {
        _module.args.pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [
            inputs.rust-overlay.overlays.default
          ];
        };

        # Add your auto-formatters here.
        # cf. https://numtide.github.io/treefmt/
        treefmt.config = {
          projectRootFile = "flake.nix";
          programs = {
            nixpkgs-fmt.enable = true;
            rustfmt.enable = true;
            leptosfmt.enable = true;
          };
        };

        leptos-fullstack.overrideCraneArgs = oa:
          let
            # 'cargo leptos test' doesn't run tests for all crates in the
            # workspace. We do it here.
            run-test = pkgs.writeShellApplication {
              name = "run-test";
              text = ''
                set -xe

                ${oa.cargoTestCommand}

                # Disable tests on macOS for https://github.com/garnix-io/issues/issues/69
                # If/when we move to Jenkins, this won't be necessary.
                ${if !pkgs.stdenv.isDarwin
                  then ''
                    # Run `cargo test` using the same settings as `cargo leptos test`
                    # In particular: target-dir and features
                    cargo test --target-dir=target/server --no-default-features --features=ssr
                    cargo test --target-dir=target/front --no-default-features --features=hydrate
                  ''
                  else ""
                }
              '';
            };
          in
          {
            nativeBuildInputs = (oa.nativeBuildInputs or [ ]) ++ [
              pkgs.nix # cargo tests need nix
            ];
            buildInputs = (oa.buildInputs or [ ]) ++ lib.optionals pkgs.stdenv.isDarwin (with pkgs.darwin.apple_sdk.frameworks; [
              IOKit
            ]);
            cargoTestCommand = lib.getExe run-test;
            meta.description = "WIP: nix-browser";
          };

        packages = {
          default = self'.packages.nix-browser;
          nix-health = config.leptos-fullstack.craneLib.buildPackage {
            inherit (config.leptos-fullstack) src;
            pname = "nix-health";
            nativeBuildInputs = [
              pkgs.nix # cargo tests need nix
            ];
            buildInputs = lib.optionals pkgs.stdenv.isDarwin (with pkgs.darwin.apple_sdk.frameworks; [
              IOKit
            ]);
            cargoExtraArgs = "-p nix_health --features ssr";
            # Disable tests on macOS for https://github.com/garnix-io/issues/issues/69
            # If/when we move to Jenkins, this won't be necessary.
            doCheck = !pkgs.stdenv.isDarwin;
          };
        };

        devShells.default = pkgs.mkShell {
          name = "nix-browser";
          inputsFrom = [
            config.treefmt.build.devShell
            self'.devShells.nix-browser
            self'.devShells.e2e-playwright
          ];
          packages = with pkgs; [
            just
            nixci
            cargo-watch
            cargo-expand
            config.process-compose.cargo-doc-live.outputs.package
          ];
          buildInputs = lib.optionals pkgs.stdenv.isDarwin (with pkgs.darwin.apple_sdk.frameworks; [
            IOKit
          ]);
          shellHook = ''
            echo
            echo "🍎🍎 Run 'just <recipe>' to get started"
            just
          '';
        };
      };
    };
}
