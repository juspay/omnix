{ inputs, ... }:
{
  imports = [
    inputs.rust-flake.flakeModules.nixpkgs
  ];
  perSystem = { inputs', config, self', pkgs, lib, system, ... }: {
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
  };
}
