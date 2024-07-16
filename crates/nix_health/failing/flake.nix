# Just a flake.nix to configure nix-health to fail all possible checks 
#
# Used for testing purposes; run as:
#   just watch-nix-health ./crates/nix_health/failing/

{
  outputs = _: {
    nix-health.default = {
      caches.required = [ "https://unknown.cachix.org" "https://example.com" ];
      nix-version.min-required = "9.99.99";
      system = {
        min_ram = "512GB";
        min_disk_space = "64TB";
      };
    };
  };
}
