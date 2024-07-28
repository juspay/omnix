{ self, pkgs, lib, flake-parts-lib, ... }:

let
  inherit (flake-parts-lib)
    mkPerSystemOption;
  cacheName = "om";
in
{
  options = {
    perSystem = mkPerSystemOption ({ config, self', pkgs, system, ... }: {
      options = {
        cache-pins.pathsToCache = lib.mkOption {
          type = lib.types.attrsOf lib.types.path;
          description = ''
            Store paths to push to a Nix cache (such as cachix)
          '';
        };
      };

      config = {
        apps.cachix-pin.program = pkgs.writeShellApplication {
          name = "cachix-pin";
          runtimeInputs = [ pkgs.cachix ];
          text = ''
            ${lib.concatStringsSep "\n" (lib.mapAttrsToList (name: path: ''
              cachix pin ${cacheName} main-${name} ${path}
              '') config.cache-pins.pathsToCache)
              }
          '';
        };
      };
    });
  };
}
