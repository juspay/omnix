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
      buildInputs = lib.optionals pkgs.stdenv.isDarwin (
        with pkgs.apple_sdk_frameworks; [
          IOKit
          CoreFoundation
        ]
      );
      NIX_FLAKE_SCHEMAS_BIN = lib.getExe pkgs.nix-flake-schemas;
      inherit (rust-project.crates."omnix-cli".crane.args)
        DEFAULT_FLAKE_SCHEMAS;
      nativeBuildInputs = with pkgs; [
        nix # Tests need nix cli
      ];
    } // lib.optionalAttrs pkgs.stdenv.isLinux {
      CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
      CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
    };
  };
}
