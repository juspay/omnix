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
  autoWire = [ ];
  crane = {
    args = {
      buildInputs = lib.optionals pkgs.stdenv.isDarwin (
        with pkgs.apple_sdk_frameworks; [
          IOKit
          CoreFoundation
        ]
      );
      inherit (rust-project.crates."nix_rs".crane.args)
        DEFAULT_FLAKE_SCHEMAS
        FLAKE_METADATA
        FLAKE_ADDSTRINGCONTEXT
        INSPECT_FLAKE
        TRUE_FLAKE
        FALSE_FLAKE
        NIX_SYSTEMS
        ;
      CACHIX_BIN = pkgs.cachix + /bin/cachix;
      nativeBuildInputs = [
        # nix # Tests need nix cli
      ];
    } // lib.optionalAttrs pkgs.stdenv.isLinux {
      CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
      CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
    };
  };
}
