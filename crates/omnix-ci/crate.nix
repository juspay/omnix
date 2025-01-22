{ flake
, pkgs
, lib
, rust-project
, ...
}:

let
  inherit (flake) inputs;
  inherit (inputs) self;
in
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
      # Disable tests due to sandboxing issues; we run them on CI
      # instead.
      doCheck = false;
      inherit (rust-project.crates."nix_rs".crane.args)
        DEFAULT_FLAKE_SCHEMAS
        FLAKE_METADATA
        FLAKE_ADDSTRINGCONTEXT
        INSPECT_FLAKE
        TRUE_FLAKE
        FALSE_FLAKE
        NIX_SYSTEMS
        ;
      inherit (rust-project.crates."omnix-health".crane.args)
        CACHIX_BIN
        ;
      DEVOUR_FLAKE = inputs.devour-flake;

      # This value is set in omnix-cli/crate.nix.
      # We use a dummy value here, however, to avoid unnecessarily rebuilding omnix-ci in CI
      OMNIX_SOURCE = pkgs.hello;
    };
  };
}
