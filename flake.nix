{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default";
    flake-parts = { url = "github:hercules-ci/flake-parts"; inputs.nixpkgs-lib.follows = "nixpkgs"; };
    dream2nix.url = "github:nix-community/dream2nix";

    # dev tools
    treefmt-nix = { url = "github:numtide/treefmt-nix"; inputs.nixpkgs.follows = "nixpkgs"; };
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;
      imports = [
        inputs.dream2nix.flakeModuleBeta
        inputs.treefmt-nix.flakeModule
        ./backend
      ];

      perSystem = { self', pkgs, lib, ... }: {
        devShells.default = pkgs.mkShell {
          inputsFrom = [
            self'.devShells.backend
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
