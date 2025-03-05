{ pkgs
, lib
, ...
}:
{
  autoWire = [ ];
  crane = {
    args = {
      nativeBuildInputs = with pkgs; with pkgs.apple_sdk_frameworks; lib.optionals stdenv.isDarwin [
        Security
        SystemConfiguration
      ] ++ [
        libiconv
        pkg-config
        nix
      ];
      buildInputs = lib.optionals pkgs.stdenv.isDarwin
        (
          with pkgs.apple_sdk_frameworks; [
            IOKit
            CoreFoundation
          ]
        ) ++ lib.optionals pkgs.stdenv.isLinux [
        pkgs.openssl
      ];
      cargoTestExtraArgs = "-- " + (lib.concatStringsSep " " [
        # requires networking
        "--skip=config::core::tests::test_config_loading"
      ]);
    };
  };
}
