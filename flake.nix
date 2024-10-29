{
  nixConfig = {
    extra-substituters = "https://om.cachix.org";
    extra-trusted-public-keys = "om.cachix.org-1:ifal/RLZJKN4sbpScyPGqJ2+appCslzu7ZZF/C01f2Q=";
  };
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";
    systems.url = "github:nix-systems/default";

    rust-flake.url = "github:juspay/rust-flake/granular-autoWire";
    rust-flake.inputs.nixpkgs.follows = "nixpkgs";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
    cachix-push.url = "github:juspay/cachix-push";

    # omnix-ci
    devour-flake.url = "github:srid/devour-flake";
    devour-flake.flake = false;
    nix-systems-x86_64-darwin.url = "github:nix-systems/x86_64-darwin";
    nix-systems-aarch64-darwin.url = "github:nix-systems/aarch64-darwin";
    nix-systems-x86_64-linux.url = "github:nix-systems/x86_64-linux";
    nix-systems-aarch64-linux.url = "github:nix-systems/aarch64-linux";

    inspect.url = "github:juspay/inspect/inventory-for-systems";
    inspect.flake = false;
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;

      # See ./nix/modules/flake-parts/*.nix for the modules that are imported here.
      imports = with builtins;
        map
          (fn: ./nix/modules/flake-parts/${fn})
          (attrNames (readDir ./nix/modules/flake-parts));
    };
}
