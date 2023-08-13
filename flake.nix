{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";

    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    treefmt-nix.url = "github:srid/treefmt-nix/leptosfmt"; # https://github.com/numtide/treefmt-nix/pull/108
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
      ];
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

        leptos-fullstack.overrideCraneArgs = oa: {
          nativeBuildInputs = (oa.nativeBuildInputs or [ ]) ++ [
            pkgs.nix # cargo tests need nix
          ];
          # Disable tests on macOS for https://github.com/garnix-io/issues/issues/69
          # If/when we move to Jenkins, this won't be necessary.
          doCheck = !pkgs.stdenv.isDarwin;
        };

        packages.default = self'.packages.nix-browser.overrideAttrs (oa: {
          installPhase = (oa.installPhase or "") + ''
            wrapProgram $out/bin/${oa.pname} \
                    --set LEPTOS_SITE_ADDR 127.0.0.1:0
          '';
        });

        devShells.default = pkgs.mkShell {
          inputsFrom = [
            config.treefmt.build.devShell
            self'.devShells.nix-browser
          ];
          packages = with pkgs; [
            just
            cargo-watch
            config.process-compose.cargo-doc-live.outputs.package
          ];
        };
      };
    };
}
