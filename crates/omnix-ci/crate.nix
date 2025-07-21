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
      # Enable tests to run via Nix  
      doCheck = true;
    };
  };
}
