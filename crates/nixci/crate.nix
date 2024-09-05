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
  autoWire = [ "doc" "clippy" ];
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
      # Disable tests due to sandboxing issues; we run them on CI
      # instead.
      doCheck = false;
      inherit (rust-project.crates."nix_rs".crane.args)
        DEVOUR_FLAKE
        DEFAULT_FLAKE_SCHEMAS
        NIX_FLAKE_SCHEMAS_BIN
        ;
      OMNIX_SOURCE = inputs.self;
    };
  };
}
