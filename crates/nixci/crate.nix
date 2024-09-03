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
  autoWire = true;
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
      DEVOUR_FLAKE = inputs.devour-flake;
      OMNIX_SOURCE = inputs.self;
      NIX_FLAKE_SCHEMAS_BIN = lib.getExe pkgs.nix-flake-schemas;
      inherit (rust-project.crates."omnix-cli".crane.args)
        DEFAULT_FLAKE_SCHEMAS;
    };
  };
}
