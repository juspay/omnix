{ lib, ... }:
{
  perSystem = { config, self', pkgs, system, ... }: {
    # e2e test service using playwright
    process-compose.e2e-playwright-test =
      let
        TEST_PORT = "5000";
      in
      {
        tui = false;
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
              runtimeInputs = with pkgs; [ nodejs playwright-test ];
              text = ''
                cd e2e
                playwright test --project chromium
              '';
            };
            depends_on."start-app".condition = "process_healthy";
            availability.exit_on_end = true;
          };
        };
      };

    devShells.e2e-playwright = pkgs.mkShell {
      buildInputs = with pkgs; [
        nodejs
        playwright-test
      ];
      shellHook = ''
        export NODE_PATH=${pkgs.playwright-test}/lib/node_modules
        # VSCode disrespects NODE_PATH https://github.com/microsoft/TypeScript/issues/8760
        # So we must manually create ./node_modules
        just node_modules NODE_PATH=$NODE_PATH
      '';
    };
  };
}
