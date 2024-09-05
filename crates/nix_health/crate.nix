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
      buildInputs = lib.optionals pkgs.stdenv.isDarwin (
        with pkgs.apple_sdk_frameworks; [
          IOKit
          CoreFoundation
        ]
      );
      inherit (rust-project.crates."nix_rs".crane.args)
        DEVOUR_FLAKE
        DEFAULT_FLAKE_SCHEMAS
        NIX_FLAKE_SCHEMAS_BIN
        ;
      nativeBuildInputs = with pkgs; [
        nix # Tests need nix cli
      ];
    } // lib.optionalAttrs pkgs.stdenv.isLinux {
      CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
      CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
    };
  };
}
