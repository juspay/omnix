{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    flake = { };
  };
  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      perSystem = { pkgs, ... }: {
        packages.default = pkgs.writeText "nix_rs-metadata.json" (builtins.toJSON {
          inputs = builtins.mapAttrs
            (k: v: v.outPath)
            (builtins.removeAttrs inputs.flake.inputs [ "self" ]);
          flake = inputs.flake.outPath;
        });
      };
    };
}
