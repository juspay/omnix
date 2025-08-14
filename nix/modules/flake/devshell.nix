let
  root = ../../..;
in
{
  imports = [
    (root + /crates/omnix-health/module/flake-module.nix)
  ];

  perSystem = { config, self', pkgs, ... }: {
    devShells.default = pkgs.mkShell {
      name = "omnix-devshell";
      meta.description = "Omnix development environment";
      inputsFrom = [
        config.pre-commit.devShell
        self'.devShells.rust
      ];
      inherit (config.rust-project.crates."omnix-cli".crane.args)
        DEVOUR_FLAKE
        NIX_SYSTEMS
        DEFAULT_FLAKE_SCHEMAS
        FLAKE_METADATA
        FLAKE_ADDSTRINGCONTEXT
        INSPECT_FLAKE
        TRUE_FLAKE
        FALSE_FLAKE
        OMNIX_SOURCE
        OM_INIT_REGISTRY
        CACHIX_BIN
        ATTIC_BIN
        ;

      packages = with pkgs; [
        just
        nixd
        bacon
        cargo-expand
        cargo-nextest
        cargo-audit
        cargo-workspaces
        trunk
      ];
    };
  };
}
