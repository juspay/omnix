{ flake
, pkgs
, lib
, ...
}:

{
  autoWire = [ ];
  crane.args = {
    buildInputs = lib.optionals pkgs.stdenv.isDarwin (
      with pkgs.apple_sdk_frameworks; [
        IOKit
      ]
    );
    OM_INIT_REGISTRY =
      lib.cleanSourceWith {
        name = "flakreate-registry";
        src = flake.inputs.self + /crates/flakreate/registry;
      };
  };
}
