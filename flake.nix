{
  description = "WIP: nix-browser";
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
      ];
      perSystem = { config, self', pkgs, lib, system, ... }:
        let
          isArch = !(builtins.isNull (builtins.match "arch" system));
          env = with pkgs; 
            (if !isArch then
              {
                CHROME_DRIVER_HEADLESS = "true";
                CHROME_BINARY_PATH = "${chromium}/bin/chromium";
                CHROME_DRIVER_URL = "http://127.0.0.1:9515";
              }
            else { }) // { TEST_PORT = "5000"; };
        in
        {
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
            meta.description = "WIP: nix-browser";
          };

          process-compose.cargo-test =
            {
              tui = false;
              port = 8975;
              settings.processes = {
                start-chromedriver = {
                  command = with pkgs; if isArch then "exit 0" else "${chromedriver}/bin/chromedriver";
                  readiness_probe = {
                    exec.command = with pkgs; builtins.toString (writeShellScript "chromedriver-health" ''
                      ${curl}/bin/curl --fail ${env.CHROME_DRIVER_URL}
                      if [ "$?" -eq 7 ]; then
                        exit 1
                      else
                        exit 0
                      fi
                    '');
                    initial_delay_seconds = 2;
                    period_seconds = 10;
                    timeout_seconds = 4;
                  };
                };
                start-app = {
                  command = "${self'.packages.default}/bin/nix-browser --site-addr 127.0.0.1:${env.TEST_PORT}";
                  readiness_probe = {
                    exec.command = with pkgs; "${curl}/bin/curl --fail 127.0.0.1:${env.TEST_PORT}";
                    initial_delay_seconds = 2;
                    period_seconds = 10;
                    timeout_seconds = 4;
                  };
                };
                test = {
                  environment = env;
                  command = if isArch then "exit 0" else "cargo leptos test";
                  depends_on."start-chromedriver".condition = "process_healthy";
                  depends_on."start-app".condition = "process_healthy";
                };
              };
            };
            
          packages.default = self'.packages.nix-browser;

          devShells.default = pkgs.mkShell ({
            inputsFrom = [
              config.treefmt.build.devShell
              self'.devShells.nix-browser
            ];
            packages = with pkgs; [
              just
              cargo-watch
              cargo-expand
              config.process-compose.cargo-doc-live.outputs.package
              config.process-compose.cargo-test.outputs.package
            ] ++ lib.optionals (!isArch) [ chromedriver chromium ];
          } // env);
        };
    };
}
