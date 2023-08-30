{ self, lib, ... }:
{
  perSystem = { config, self', inputs', pkgs, system, ... }:
    lib.mkIf (system == "x86_64-linux")
      {
        # e2e test service using rust (thirty-four)
        # only supports "x86_64-linux" system
        process-compose.cargo-e2e-test =
          let
            env = {
              CHROME_DRIVER_HEADLESS = "true";
              CHROME_BINARY_PATH = lib.getExe pkgs.chromium;
              CHROME_DRIVER_URL = "http://127.0.0.1:9515";
              TEST_PORT = "5000";
            };
          in
          {
            tui = false;
            port = 8975;
            settings.processes = {
              start-chromedriver = {
                command = lib.getExe pkgs.chromedriver;
                readiness_probe = {
                  exec.command = builtins.toString (pkgs.writeShellScript "chromedriver-health" ''
                    ${lib.getExe pkgs.curl} --fail ${env.CHROME_DRIVER_URL}
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
                command = "${lib.getExe self'.packages.default} --site-addr=127.0.0.1:${env.TEST_PORT}";
                readiness_probe = {
                  exec.command = "${lib.getExe pkgs.curl} --fail 127.0.0.1:${env.TEST_PORT}";
                  initial_delay_seconds = 2;
                  period_seconds = 10;
                  timeout_seconds = 4;
                };
              };
              test = {
                environment = env;
                command = "cargo test -- --include-ignored";
                depends_on."start-chromedriver".condition = "process_healthy";
                depends_on."start-app".condition = "process_healthy";
              };
            };
          };
      };
}
