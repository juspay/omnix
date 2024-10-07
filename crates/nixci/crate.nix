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
        DEFAULT_FLAKE_SCHEMAS
        INSPECT_FLAKE
        NIX_SYSTEMS
        ;
      DEVOUR_FLAKE = inputs.devour-flake;

      # To avoid unnecessary rebuilds, start from cleaned source, and then add the Nix files necessary to `nix run` it. Finally, add any files required by the Rust build.
      OMNIX_SOURCE = lib.cleanSourceWith {
        src = inputs.self;
        filter = path: type:
          rust-project.crane-lib.filterCargoSources path type
          || lib.hasSuffix ".nix" path
          || lib.hasSuffix "flake.lock" path
          || lib.hasSuffix "registry.json" path
        ;
      };
    };
  };
}
