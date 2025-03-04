{ pkgs
, lib
, ...
}:
{
  autoWire = [ ];
  crane = {
    args = {
      buildInputs = lib.optionals pkgs.stdenv.isDarwin (
        with pkgs.apple_sdk_frameworks; [
          IOKit
          CoreFoundation
        ]
      );
      nativeBuildInputs = [
        # nix # Tests need nix cli
      ];
    };
  };
}
