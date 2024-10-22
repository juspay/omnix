{ inputs, ... }:
{
  imports = [
    inputs.cachix-push.flakeModule
  ];

  perSystem = { self', ... }: {
    cachix-push = {
      cacheName = "om";
      # https://om.cachix.org will ensure that these paths are always
      # available. The rest may be be GC'ed.
      pathsToCache = {
        cli = self'.packages.default;
      };
    };
  };
}
