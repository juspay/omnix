{ flake
, pkgs
, lib
, ...
}:

let
  inherit (flake) inputs;
in
{
  autoWire = true;
  crane.args = {
    buildInputs = lib.optionals pkgs.stdenv.isDarwin (
      with pkgs.apple_sdk_frameworks; [
        IOKit
      ]
    );
    DEFAULT_FLAKE_SCHEMAS = lib.cleanSourceWith {
      name = "flake-schemas";
      src = flake.inputs.self + /nix/flake-schemas;
    };
    DEVOUR_FLAKE = inputs.devour-flake;
    OMNIX_SOURCE = inputs.self;
    NIX_FLAKE_SCHEMAS_BIN = lib.getExe pkgs.nix-flake-schemas;
  };
}
