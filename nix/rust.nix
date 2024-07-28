{ inputs, ... }:
# Nix module for the Rust part of the project
#
# This uses Crane, via https://github.com/juspay/rust-flake
{
  perSystem = { config, self', pkgs, lib, system, ... }:
    let
      apple_sdk_frameworks =
        if system == "x86_64-darwin"
        # To compile for `x86_64-darwin` we need 11.0
        # see: https://github.com/NixOS/nixpkgs/pull/261683#issuecomment-1772935802
        then pkgs.darwin.apple_sdk_11_0.frameworks
        else pkgs.darwin.apple_sdk.frameworks;
    in
    {
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
                nativeBuildInputs = with pkgs; with apple_sdk_frameworks; lib.optionals stdenv.isDarwin [
                  Security
                  SystemConfiguration
                ] ++ [
                  libiconv
                  pkg-config
                ];
                buildInputs = lib.optionals pkgs.stdenv.isDarwin
                  (
                    with apple_sdk_frameworks; [
                      IOKit
                      CoreFoundation
                    ]
                  ) ++ lib.optionals pkgs.stdenv.isLinux [
                  pkgs.pkgsStatic.openssl
                ];
                DEVOUR_FLAKE = inputs.devour-flake;
                OM_INIT_REGISTRY = inputs.self + /crates/flakreate/registry;
                # Disable tests due to sandboxing issues; we run them on CI
                # instead.
                doCheck = false;
                meta = {
                  description = "Command-line interface for Omnix";
                  mainProgram = "om";
                };
              } //
              lib.optionalAttrs pkgs.stdenv.isLinux {
                CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
                CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
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
                  with apple_sdk_frameworks; [
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
                meta.description = "Graphical user interface for Omnix";
              };
            };
          };
          "nix_rs" = {
            autoWire = true;
            crane = {
              args = {
                buildInputs = lib.optionals pkgs.stdenv.isDarwin (
                  with apple_sdk_frameworks; [
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
                  with apple_sdk_frameworks; [
                    IOKit
                    CoreFoundation
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
          "nixci" = {
            crane = {
              args = {
                nativeBuildInputs = with pkgs; with apple_sdk_frameworks; lib.optionals stdenv.isDarwin [
                  Security
                  SystemConfiguration
                ] ++ [
                  libiconv
                  pkg-config
                ];
                buildInputs = lib.optionals pkgs.stdenv.isDarwin
                  (
                    with apple_sdk_frameworks; [
                      IOKit
                      CoreFoundation
                    ]
                  ) ++ lib.optionals pkgs.stdenv.isLinux [
                  pkgs.openssl
                ];
                DEVOUR_FLAKE = inputs.devour-flake;
              };
            };
          };
          "flakreate" = {
            crane.args = {
              buildInputs = lib.optionals pkgs.stdenv.isDarwin (
                with apple_sdk_frameworks; [
                  IOKit
                ]
              );
            };
          };
        };

        src = lib.cleanSourceWith {
          name = "omnix-project-root";
          src = inputs.self; # The original, unfiltered source
          filter = path: type:
            # TODO: This should be applied for omnix-gui crate only (via rust-flake)
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
          default = crates."omnix-cli".crane.outputs.drv.crate.overrideAttrs (oa: {
            nativeBuildInputs = (oa.nativeBuildInputs or [ ]) ++ [ pkgs.installShellFiles ];
            postInstall = ''
              installShellCompletion --cmd om \
                --bash <($out/bin/om completion bash) \
                --zsh <($out/bin/om completion zsh) \
                --fish <($out/bin/om completion fish)
            '';
          });
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
          nix-health = self'.packages.nix_health;
        };

      cargo-doc-live.crateName = "omnix-gui";
    };
}
