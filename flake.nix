{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default";
    flake-parts = { url = "github:hercules-ci/flake-parts"; inputs.nixpkgs-lib.follows = "nixpkgs"; };
    treefmt-nix = { url = "github:numtide/treefmt-nix"; inputs.nixpkgs.follows = "nixpkgs"; };
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;
      imports = [ inputs.treefmt-nix.flakeModule ];

      perSystem = { self', pkgs, lib, ... }:
        let
          src = lib.sourceFilesBySuffices inputs.self [ ".rs" ".toml" "Cargo.lock" ];
          inherit (lib.importTOML (src + "/Cargo.toml")) package;
        in
        {
          packages = {
            ${package.name} = pkgs.rustPlatform.buildRustPackage {
              pname = package.name;
              inherit (package) version;
              inherit src;
              cargoLock.lockFile = (src + "/Cargo.lock");
            };
            default = self'.packages.${package.name};
          };

          devShells.default = pkgs.mkShell {
            inherit (package) name;
            inputsFrom = [ self'.packages.${package.name} ];
            packages = with pkgs; [
              rust-analyzer
              nil
            ];
          };

          # Run `nix fmt` to format the source tree.
          treefmt = {
            projectRootFile = "flake.nix";
            programs.nixpkgs-fmt.enable = true;
            programs.rustfmt.enable = true;
          };
        };
    };
}
