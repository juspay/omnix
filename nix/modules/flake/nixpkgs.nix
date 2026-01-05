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
