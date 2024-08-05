{ inputs, ... }:

{
  perSystem = { pkgs, ... }: {
    packages.doc = pkgs.callPackage (inputs.self + /doc) { };
  };
}
