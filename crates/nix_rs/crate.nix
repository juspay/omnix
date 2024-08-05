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
  crane = {
    args = {
      buildInputs = lib.optionals pkgs.stdenv.isDarwin (
        with pkgs.apple_sdk_frameworks; [
          IOKit
        ]
      );
      nativeBuildInputs = with pkgs; [
        nix # Tests need nix cli
      ];
      NIX_FLAKE_SCHEMAS_BIN = lib.getExe pkgs.nix-flake-schemas;
      DEFAULT_FLAKE_SCHEMAS = inputs.flake-schemas;
    };
  };
}
