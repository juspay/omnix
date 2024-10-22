# Just a flake.nix to configure omnix-health to fail all possible checks
#
# Used for testing purposes; run as:
#   cargo run -p omnix-cli health ./crates/omnix-health/failing

{
  outputs = _: {
    om.health.default = {
      caches.required = [ "https://unknown.cachix.org" "https://example.com" ];
      nix-version.min-required = "9.99.99";
      system = {
        min_ram = "512GB";
        min_disk_space = "64TB";
      };
    };
  };
}
