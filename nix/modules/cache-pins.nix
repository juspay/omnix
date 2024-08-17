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
        apps.cachix-push.program = pkgs.writeShellApplication {
          name = "cachix-push";
          meta.description = ''
            Run `cachix push` & `cachix pin` for each path in `cache-pins.pathsToCache`
          '';
          runtimeInputs = [ pkgs.cachix ];
          text = ''
            set -x
            ${lib.concatStringsSep "\n" (lib.mapAttrsToList (name: path: ''
              cachix push ${cacheName} ${path}
              cachix pin ${cacheName} ${name}-${system} ${path}
              '') config.cache-pins.pathsToCache)
              }
          '';
        };
      };
    });
  };

  config = {
    perSystem = { self', ... }: {
      # https://om.cachix.org will ensure that these paths are always
      # available. The rest may be be GC'ed.
      cache-pins.pathsToCache = {
        cli = self'.packages.default;
      };
    };
  };
}
