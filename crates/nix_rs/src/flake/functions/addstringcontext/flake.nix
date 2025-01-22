{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    jsonfile = { flake = false; };
  };
  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      perSystem = { pkgs, lib, ... }:
        let
          json = builtins.fromJSON (builtins.readFile inputs.jsonfile);
          jsonWithPathContext = lib.flip lib.mapAttrsRecursive json (k: v:
            if lib.lists.last k == "outPaths" then
              builtins.map (path: builtins.storePath path) v
            else
              v
          );
        in
        {
          packages.default = pkgs.writeText "addstringcontext.json" (builtins.toJSON jsonWithPathContext);
        };
    };
}
