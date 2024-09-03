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
  autoWire = false;
  crane = {
    args = {
      buildInputs = lib.optionals pkgs.stdenv.isLinux
        (with pkgs; [
          webkitgtk_4_1
          xdotool
          pkg-config
        ]) ++ lib.optionals pkgs.stdenv.isDarwin (
        with pkgs.apple_sdk_frameworks; [
          IOKit
          Carbon
          WebKit
          Security
          Cocoa
          CoreFoundation
        ]
      );
      nativeBuildInputs = with pkgs;[
        pkg-config
        makeWrapper
        tailwindcss
        dioxus-cli
        pkgs.nix # cargo tests need nix
      ];
      NIX_FLAKE_SCHEMAS_BIN = lib.getExe pkgs.nix-flake-schemas;
      inherit (rust-project.crates."omnix-cli".crane.args)
        DEFAULT_FLAKE_SCHEMAS;
      meta.description = "Graphical user interface for Omnix";
    };
  };
}
