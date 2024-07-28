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
            Store paths to push to/pin in a Nix cache (such as cachix)
          '';
        };
      };

      config = {
        apps.cachix-pin.program = pkgs.writeShellApplication {
          name = "cachix-pin";
          meta.description = ''
            Run `cachix pin` for each path in `cache-pins.pathsToCache`
          '';
          runtimeInputs = [ pkgs.cachix ];
          text = ''
            set -x
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
