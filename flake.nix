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
    process-compose-flake.url = "github:Platonic-Systems/process-compose-flake";
    cargo-doc-live.url = "github:srid/cargo-doc-live";
    cachix-push.url = "github:juspay/cachix-push";

    # nixci
    devour-flake.url = "github:srid/devour-flake";
    devour-flake.flake = false;
    nix-systems-x86_64-darwin.url = "github:nix-systems/x86_64-darwin";
    nix-systems-aarch64-darwin.url = "github:nix-systems/aarch64-darwin";
    nix-systems-x86_64-linux.url = "github:nix-systems/x86_64-linux";
    nix-systems-aarch64-linux.url = "github:nix-systems/aarch64-linux";

    # TODO: Use upstream after https://github.com/NixOS/nix/pull/8892
    # Note: This version of nix is only used to run `nix flake show` in omnix-cli
    # Also note: Using shivaraj-bh fork of nix which fixes x86_64-darwin on top of github:DeterminateSystems/nix-src/flake-schemas
    nix.url = "github:shivaraj-bh/nix/flake-schemas";
    nix.inputs.flake-parts.follows = "flake-parts";
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;

      # See ./nix/modules/*.nix for the modules that are imported here.
      imports = with builtins;
        map
          (fn: ./nix/modules/${fn})
          (attrNames (readDir ./nix/modules));
    };
}
