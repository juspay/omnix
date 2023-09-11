{ lib, ... }:
{
  perSystem = { config, self', pkgs, system, ... }: {
    # e2e test service using playwright
    process-compose.e2e-playwright-test =
      let
        TEST_PORT = "5000";
        PLAYWRIGHT_BROWSERS_PATH = "${pkgs.playwright-driver.browsers}";
      in
      {
        tui = false;
        port = 8975;
        settings.processes = {
          start-app = {
            command = "${lib.getExe self'.packages.default} --site-addr=127.0.0.1:${TEST_PORT} --no-open";
            readiness_probe = {
              exec.command = "${lib.getExe pkgs.curl} --fail 127.0.0.1:${TEST_PORT}";
              initial_delay_seconds = 2;
              period_seconds = 10;
              timeout_seconds = 4;
            };
          };
          test = {
            environment = {
              inherit TEST_PORT;
            };
            command = pkgs.writeShellApplication {
              name = "e2e-playwright";
              runtimeInputs = with pkgs; [ nodejs playwright-test PLAYWRIGHT_BROWSERS_PATH ];
              text = ''
                cd e2e-playwright
                playwright test --project chromium
              '';
            };
            depends_on."start-app".condition = "process_healthy";
            availability.exit_on_end = true;
          };
        };
      };

    devShells.e2e-playwright =
      let
        nodeModules = pkgs.buildNpmPackage {
          pname = "e2e-playwright";
          version = "1.0.0";
          src = builtins.path {
            path = ./.;
            filter = path: _:
              let name = builtins.baseNameOf path; in
              name == "package.json" ||
              name == "package-lock.json" ||
              name == "tests" ||
              name == "playwright.config.js";
          };
          npmDepsHash = "sha256-hmZhH2H2Sx/YcCj2dALDM2VX0KeD0eg7xrRw4wI6klQ=";
          # npmDepsHash = lib.fakeHash;
          dontBuild = true;
          installPhase = ''
            cp -r node_modules $out/
          '';
        };
      in
      pkgs.mkShell {
        buildInputs = with pkgs; [
          nodejs
          playwright-test
        ];
        shellHook = ''
          export NODE_PATH=${nodeModules}
          # VSCode disrespects NODE_PATH https://github.com/microsoft/TypeScript/issues/8760
          # So we must manually create ./node_modules
          just node_modules NODE_PATH=$NODE_PATH
        '';
      };
  };
}
