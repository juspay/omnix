{
  # omnix configuration
  flake = {
    om = {
      ci.default = {
        omnix = {
          dir = ".";
          steps = {
            # build.enable = false;
            flake-check.enable = false; # Not necessary
            custom = {
              om-show = {
                type = "app";
                # name = "default";
                args = [ "show" "." ];
              };
              build-omnix-source = {
                type = "app";
                name = "build-omnix-source";
              };
              binary-size-is-small = {
                type = "app";
                name = "check-closure-size";
                systems = [ "x86_64-linux" ]; # We have static binary for Linux only.
              };
              cargo-tests = {
                type = "devshell";
                # name = "default";
                command = [ "cargo" "test" ];
                systems = [ "x86_64-linux" "aarch64-darwin" ]; # Too slow on rosetta
              };
            };
          };
        };
        doc.dir = "doc";

        registry = {
          dir = "crates/omnix-init/registry";
          steps = {
            build.enable = false;

            # FIXME: Why does omnix require this?
            custom = { };
          };
        };

        # Because the cargo tests invoking Nix doesn't pass github access tokens..
        # To avoid GitHub rate limits during the integration test (which
        # doesn't use the token)
        cli-test-dep-cache = {
          dir = "crates/omnix-cli/tests";
          steps = {
            lockfile.enable = false;
            flake_check.enable = false;
            # FIXME: Why does omnix require this?
            custom = { };
          };
        };
      };
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
  };
}
