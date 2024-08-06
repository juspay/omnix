{
  outputs = _: {
    flakeModules = rec {
      default = om;
      om = ./flake-module.nix;
    };
  };
}
