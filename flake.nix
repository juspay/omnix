{
  nixConfig = {
    extra-substituters = "https://om.cachix.org";
    extra-trusted-public-keys = "om.cachix.org-1:ifal/RLZJKN4sbpScyPGqJ2+appCslzu7ZZF/C01f2Q=";
  };
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";

    rust-flake.url = "github:juspay/rust-flake";
    rust-flake.inputs.nixpkgs.follows = "nixpkgs";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
    process-compose-flake.url = "github:Platonic-Systems/process-compose-flake";
    cargo-doc-live.url = "github:srid/cargo-doc-live";
    cachix-push.url = "github:juspay/cachix-push";

    devour-flake.url = "github:srid/devour-flake";
    devour-flake.flake = false;

    # TODO: Use juspay/flake-schemas
    flake-schemas.url = "github:shivaraj-bh/flake-schemas/om-schema";
    flake-schemas.flake = false;
    # TODO: Use upstream after https://github.com/NixOS/nix/pull/8892
    # Note: This version of nix is only used to run `nix flake show` in omnix-cli
    # Also note: Using shivaraj-bh fork of nix which fixes x86_64-darwin on top of github:DeterminateSystems/nix-src/flake-schemas
    nix.url = "github:shivaraj-bh/nix/flake-schemas";

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
