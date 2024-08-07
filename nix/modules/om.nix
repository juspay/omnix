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
