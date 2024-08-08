# The omnix flake-parts module
{ self, lib, flake-parts-lib, ... }:

let
  inherit (flake-parts-lib)
    mkPerSystemOption;
in
{
  options = {
    perSystem = mkPerSystemOption
      ({ config, pkgs, ... }: {
        options.om.health.outputs.devShell = lib.mkOption {
          type = lib.types.package;
          description = ''
            Add a shellHook for running `om health` on the flake.
          '';
          default = pkgs.mkShell {
            shellHook = ''
              # Must use a subshell so that 'trap' handles only `om health`
              # crashes.
              (
                trap "${lib.getExe pkgs.toilet} NIX UNHEALTHY --filter gay -f smmono9" ERR

                ${lib.getExe pkgs.omnix} health --quiet .
              )
            '';
          };
        };
      });
  };
}
