{ pkgs, lib, ... }:
{
  # Autowiring `crate` so that the tests are run in nix sandbox when `om ci` is used
  autoWire = [ "crate" ];
  crane = {
    args = {
      buildInputs = lib.optionals pkgs.stdenv.isDarwin (
        with pkgs.apple_sdk_frameworks; [
          IOKit
          SystemConfiguration
        ]
      );
      nativeBuildInputs = with pkgs; [
        nix # Tests need nix cli
      ];
    };
  };
}
