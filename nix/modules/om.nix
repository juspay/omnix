{ inputs, ... }:

let
  root = inputs.self;
in
{
  # omnix configuration
  flake = {
    om = {
      ci.default = {
        omnix.dir = root;
        flakreate-registry.dir = root + /crates/flakreate/registry;
        doc.dir = root + /doc;

        # Because the cargo tests invoking Nix doesn't pass github access tokens..
        # To avoid GitHub rate limits during the integration test (which
        # doesn't use the token)
        cli-test-dep-cache.dir = root + /crates/omnix-cli/tests;
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
