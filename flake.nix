{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";
    nixpkgs-leptosfmt.url = "github:nixos/nixpkgs/697d536087725659f0e047918b57082dcc5e258a"; # TODO: remove after https://nixpk.gs/pr-tracker.html?pr=248148

    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    process-compose-flake.url = "github:Platonic-Systems/process-compose-flake";

    leptos-fullstack.url = "github:srid/leptos-fullstack";
    leptos-fullstack.flake = false;
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;
      imports = [
        inputs.treefmt-nix.flakeModule
        inputs.process-compose-flake.flakeModule
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
          };
        };

        process-compose.cargo-docs-server =
          let
            port = builtins.toString 8008;
            browser-sync = lib.getExe pkgs.nodePackages.browser-sync;
            crateName = builtins.replaceStrings [ "-" ] [ "_" ]
              ((lib.trivial.importTOML ./Cargo.toml).package.name);
          in
          {
            tui = false;
            port = 8974; # process-compose exits silently if port is in use; set this to something uniqiue (hopefully)
            settings.processes = {
              cargo-doc.command = builtins.toString (pkgs.writeShellScript "cargo-doc" ''
                run-cargo-doc() {
                  cargo doc --document-private-items --all-features
                  ${browser-sync} reload --port ${port}  # Trigger reload in browser
                }; export -f run-cargo-doc
                cargo watch -s run-cargo-doc
              '');
              browser-sync.command = ''
                ${browser-sync} start --port ${port} --ss target/doc -s target/doc \
                  --startPath /${crateName}/
              '';
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
            config.process-compose.cargo-docs-server.outputs.package
            inputs.nixpkgs-leptosfmt.legacyPackages.${system}.leptosfmt
          ];
        };
      };
    };
}
