{ flake
, pkgs
, lib
, rust-project
, ...
}:

let
  inherit (flake) inputs;
in
{
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
      DEVOUR_FLAKE = inputs.devour-flake;
      DEFAULT_FLAKE_SCHEMAS = lib.cleanSourceWith {
        name = "flake-schemas";
        src = flake.inputs.self + /nix/flake-schemas;
      };
      NIX_FLAKE_SCHEMAS_BIN = lib.getExe pkgs.nix-flake-schemas;
    };
  };
}
