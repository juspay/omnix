# TODO: This should be a proper flake-parts module with options
{ inputs, ... }:

{
  flake = {
    # jq < $(nix eval --raw .#registry)
    flakeEvalCache =
      let
        attrsToInclude = [ "om" ];
      in
      builtins.toFile "flake.nix.json"
        (builtins.toJSON
          (inputs.nixpkgs.lib.filterAttrs
            (name: _: builtins.elem name attrsToInclude)
            inputs.self));
  };

  perSystem = { self', pkgs, lib, ... }: {
    packages.cache = pkgs.writeShellApplication {
      name = "write-flake-cache";
      text = ''
        set -x
        cp ${inputs.self.flakeEvalCache} ./flake.nix.json
      '';
    };
    apps.cache.program = self'.packages.cache;
    checks.cache = pkgs.runCommandNoCC "check-flake-cache" { } ''
      # Check that flake.nix.json is up to date with that `nix run .#cache` generates
      OLD=$(cat ${inputs.self}/flake.nix.json)
      ${lib.getExe self'.packages.cache}
      NEW=$(cat flake.nix.json)
      if [ "$OLD" != "$NEW" ]; then
        echo "ERROR: flake.nix.json is out of date. Run 'nix run .#cache' to update it."
        exit 1
      fi
      touch $out
    '';
  };
}
