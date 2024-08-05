{ inputs, ... }:

{
  # omnix configuration
  flake = {
    om = {
      ci.default = {
        omnix.dir = inputs.self;
        flakreate-registry.dir = inputs.self + /crates/flakreate/registry;
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
