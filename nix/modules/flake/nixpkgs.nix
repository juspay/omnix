{ inputs, ... }:
{
  imports = [
    inputs.rust-flake.flakeModules.nixpkgs
  ];
  perSystem = { inputs', config, self', pkgs, lib, system, ... }: {
    nixpkgs.overlays = [
      (self: super: {
        # Configure tailwind to enable all relevant plugins
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

        # To compile for `x86_64-darwin` we need 11.0
        # see: https://github.com/NixOS/nixpkgs/pull/261683#issuecomment-1772935802
        apple_sdk_frameworks =
          if system == "x86_64-darwin"
          then pkgs.darwin.apple_sdk_11_0.frameworks
          else pkgs.darwin.apple_sdk.frameworks;

        # Like pkgs.nix, but with flake-schemas implemented.
        # Using until https://github.com/NixOS/nix/pull/8892 is upstreamed
        nix-flake-schemas =
          if pkgs.stdenv.isLinux
          then inputs'.nix.packages.nix-static
          else inputs'.nix.packages.default;
      })
    ];
  };
}
