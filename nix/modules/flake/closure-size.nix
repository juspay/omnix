{ ... }:

let

  maxSize = 1000000 * maxSizeInMB.total;
  maxSizeInMB = rec {
    total = omnix + cachix;
    omnix = 60;
    cachix = 150;
  };
in
{
  perSystem = { pkgs, ... }: {
    apps.check-closure-size = rec {
      meta.description = program.meta.description;
      program = pkgs.writeShellApplication {
        name = "omnix-check-closure-size";
        runtimeInputs = [ pkgs.jq pkgs.bc pkgs.nix ];
        meta.description = "Check that omnix's nix closure size remains reasonably small";
        text = ''
          set -o pipefail
          MAX_CLOSURE_SIZE=${builtins.toString maxSize}
          CLOSURE_SIZE=$(nix path-info --json -S .#default | jq '.[].closureSize')
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
    };
  };
}
