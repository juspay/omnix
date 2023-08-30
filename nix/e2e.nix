{ self, lib, ... }:
    {
        perSystem = {}: 
            {
                process-compose.cargo-test = 
                let
                    isDarwinOrArch = with pkgs; stdenv.isDarwin || !(builtins.isNull (builtins.match "aarch.*" system));
                    env = with pkgs; {
                        CHROME_DRIVER_HEADLESS = "true";
                        CHROME_BINARY_PATH = with pkgs; if isDarwinOrArch then "" else "${chromium}/bin/chromium";
                        CHROME_DRIVER_URL = "http://127.0.0.1:9515";
                        TEST_PORT = "5000";
                    };
                in {
                    tui = false;
                    port = 8975;
                    settings.processes = {
                        start-chromedriver = {
                        command = if isDarwinOrArch then "exit 0" else with pkgs; "${chromedriver}/bin/chromedriver";
                        readiness_probe = {
                            exec.command = if isDarwinOrArch then "exit 0" else
                            with pkgs; builtins.toString (writeShellScript "chromedriver-health" ''
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
                        command = "${self'.packages.default}/bin/nix-browser --site-addr=127.0.0.1:${env.TEST_PORT}";
                        readiness_probe = {
                            exec.command = with pkgs; "${curl}/bin/curl --fail 127.0.0.1:${env.TEST_PORT}";
                            initial_delay_seconds = 2;
                            period_seconds = 10;
                            timeout_seconds = 4;
                        };
                        };
                        test = {
                        environment = env;
                        command = if isDarwinOrArch then "cargo test" else "cargo test -- --include-ignored";
                        depends_on."start-chromedriver".condition = "process_healthy";
                        depends_on."start-app".condition = "process_healthy";
                        };
                    };
                };
            }
    }