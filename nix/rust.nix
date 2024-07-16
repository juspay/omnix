{ inputs, ... }:
# Nix module for the Rust part of the project
#
# This uses Crane, via https://github.com/juspay/rust-flake
{
  perSystem = { config, self', pkgs, lib, system, ... }: {
    nixpkgs.overlays = [
      # Configure tailwind to enable all relevant plugins
      (self: super: {
        tailwindcss = super.tailwindcss.overrideAttrs
          (oa: {
            plugins = [
              pkgs.nodePackages."@tailwindcss/aspect-ratio"
              pkgs.nodePackages."@tailwindcss/forms"
              pkgs.nodePackages."@tailwindcss/language-server"
              pkgs.nodePackages."@tailwindcss/line-clamp"
              pkgs.nodePackages."@tailwindcss/typography"
            ];
          });
      })
    ];

    rust-project = {
      crates = {
        "omnix-cli" = {
          autoWire = false;
          crane = {
            args = {
              buildInputs = lib.optionals pkgs.stdenv.isLinux
                (with pkgs; [
                  pkg-config
                ]) ++ lib.optionals pkgs.stdenv.isDarwin (
                with pkgs.darwin.apple_sdk.frameworks; [
                  IOKit
                ]
              );
              nativeBuildInputs = with pkgs;[
                pkg-config
              ];
              meta = {
                description = "Command-line interface for Omnix";
                mainProgram = "om";
              };
            };
          };
        };
        "omnix-gui" = {
          autoWire = false;
          crane = {
            args = {
              buildInputs = lib.optionals pkgs.stdenv.isLinux
                (with pkgs; [
                  webkitgtk_4_1
                  xdotool
                  pkg-config
                ]) ++ lib.optionals pkgs.stdenv.isDarwin (
                with pkgs.darwin.apple_sdk.frameworks; [
                  IOKit
                  Carbon
                  WebKit
                  Security
                  Cocoa
                  # Use newer SDK because some crates require it
                  # cf. https://github.com/NixOS/nixpkgs/pull/261683#issuecomment-1772935802
                  pkgs.darwin.apple_sdk_11_0.frameworks.CoreFoundation
                ]
              );
              nativeBuildInputs = with pkgs;[
                pkg-config
                makeWrapper
                tailwindcss
                dioxus-cli
                pkgs.nix # cargo tests need nix
              ];
              meta.description = "Graphical user interface for Omnix";
            };
          };
        };
        "nix_rs" = {
          autoWire = true;
          crane = {
            args = {
              buildInputs = lib.optionals pkgs.stdenv.isDarwin (
                with pkgs.darwin.apple_sdk.frameworks; [
                  IOKit
                ]
              );
              nativeBuildInputs = with pkgs; [
                nix # Tests need nix cli
              ];
            };
          };
        };
        "nix_health" = {
          autoWire = true;
          crane = {
            args = {
              buildInputs = lib.optionals pkgs.stdenv.isDarwin (
                with pkgs.darwin.apple_sdk.frameworks; [
                  IOKit
                  # apple_sdk refers to SDK version 10.12. To compile for `x86_64-darwin` we need 11.0
                  # see: https://github.com/NixOS/nixpkgs/pull/261683#issuecomment-1772935802
                  pkgs.darwin.apple_sdk_11_0.frameworks.CoreFoundation
                ]
              );
              nativeBuildInputs = with pkgs; [
                nix # Tests need nix cli
              ];
              meta.mainProgram = "nix-health";
            } // lib.optionalAttrs pkgs.stdenv.isLinux {
              CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
              CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
            };
          };
        };
      };

      src = lib.cleanSourceWith {
        src = inputs.self; # The original, unfiltered source
        filter = path: type:
          (lib.hasSuffix "\.html" path) ||
          (lib.hasSuffix "tailwind.config.js" path) ||
          # Example of a folder for images, icons, etc
          (lib.hasInfix "/assets/" path) ||
          (lib.hasInfix "/css/" path) ||
          # Default filter from crane (allow .rs files)
          (config.rust-project.crane-lib.filterCargoSources path type)
        ;
      };
    };

    packages =
      let
        inherit (config.rust-project) crates;
      in
      {
        default = crates."omnix-cli".crane.outputs.drv.crate;
        gui = crates."omnix-gui".crane.outputs.drv.crate.overrideAttrs (oa: {
          # Copy over assets for the desktop app to access
          installPhase =
            (oa.installPhase or"") + ''
              cp -r ./crates/omnix-gui/assets/* $out/bin/
            '';
          postFixup =
            (oa.postFixup or"") + ''
              # HACK: The Linux desktop app is unable to locate the assets
              # directory, but it does look into the current directory.
              # So, `cd` to the directory containing assets (which is
              # `bin/`, per the installPhase above) before launching the
              # app.
              wrapProgram $out/bin/${ oa. pname} \
                --chdir $out/bin
            '';
        });
      };

    cargo-doc-live.crateName = "omnix-gui";
  };
}
