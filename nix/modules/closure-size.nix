{ ... }:

let
  maxSizeInMB = 60;
in
{
  perSystem = { pkgs, ... }: {
    apps.check-closure-size = {
      program = pkgs.writeShellApplication {
        name = "omnix-check-closure-size";
        runtimeInputs = [ pkgs.jq pkgs.bc pkgs.nix ];
        meta.description = "Check that omnix's nix closure size remains reasonably small";
        text = ''
          set -o pipefail
          MAX_CLOSURE_SIZE=$(echo "${builtins.toString maxSizeInMB} * 1000000" | bc)
          CLOSURE_SIZE=$(nix path-info --json -S .#default | jq '.[0]'.closureSize)
          echo "Omnix closure size: $CLOSURE_SIZE"
          echo "    Max closure size: $MAX_CLOSURE_SIZE"
          if [ "$CLOSURE_SIZE" -gt "$MAX_CLOSURE_SIZE" ]; then
              echo "ERROR: Omnix's nix closure size has increased"
              exit 3
          else
              echo "OK: Omnix's nix closure size is within limits"
          fi
        '';
      };
      meta.description = "Check that omnix's nix closure size remains reasonably small";
    };
  };
}
