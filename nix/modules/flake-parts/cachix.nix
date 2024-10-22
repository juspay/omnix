# Provide cachix as a flake app for convenient use when the user may not already have it installed yet they are expected to `nix run` omnix.
{
  perSystem = { pkgs, ... }: {
    packages.cachix = pkgs.cachix;
  };
}
