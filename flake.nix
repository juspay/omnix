{
  nixConfig = {
    extra-substituters = "https://cache.nixos.asia/oss";
    extra-trusted-public-keys = "oss:KO872wNJkCDgmGN3xy9dT89WAhvv13EiKncTtHDItVU=";
  };
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";
    systems.url = "github:nix-systems/default";

    rust-flake.url = "github:juspay/rust-flake";
    rust-flake.inputs.nixpkgs.follows = "nixpkgs";
    git-hooks.url = "github:cachix/git-hooks.nix";
    git-hooks.flake = false;

    # We cache these inputs locally, and then have Omnix reference the catched inputs.
    devour-flake.url = "github:srid/devour-flake";
    devour-flake.flake = false;
    nix-systems-x86_64-darwin.url = "github:nix-systems/x86_64-darwin";
    nix-systems-aarch64-darwin.url = "github:nix-systems/aarch64-darwin";
    nix-systems-x86_64-linux.url = "github:nix-systems/x86_64-linux";
    nix-systems-aarch64-linux.url = "github:nix-systems/aarch64-linux";
    true.url = "github:boolean-option/true";
    false.url = "github:boolean-option/false";
    inspect.url = "github:juspay/inspect/inventory-for-systems";
    inspect.flake = false;
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;
      debug = true;

      # See ./nix/modules/flake/*.nix for the modules that are imported here.
      imports = with builtins;
        map
          (fn: ./nix/modules/flake/${fn})
          (attrNames (readDir ./nix/modules/flake));
    };
}
