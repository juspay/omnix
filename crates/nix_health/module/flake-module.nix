{ self, lib, flake-parts-lib, ... }:

let
  inherit (flake-parts-lib)
    mkPerSystemOption;
in
{
  options = {
    perSystem = mkPerSystemOption
      ({ config, pkgs, ... }: {
        options.nix-health.outputs.devShell = lib.mkOption {
          type = lib.types.package;
          description = ''
            Add a shellHook for running nix-health on the flake.
          '';
          default = pkgs.mkShell {
            shellHook = ''
              # Must use a subshell so that 'trap' handles only nix-health
              # crashes.
              (
                trap "${lib.getExe pkgs.toilet} NIX UNHEALTHY --filter gay -f smmono9" ERR

                ${lib.getExe pkgs.nix-health} --quiet .
              )
            '';
          };
        };
      });
  };
}
