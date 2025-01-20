{ flake
, rust-project
, pkgs
, lib
, ...
}:

{
  autoWire = [ ];
  crane = {
    args = {
      inherit (rust-project.crates."nix_rs".crane.args)
        DEFAULT_FLAKE_SCHEMAS
        FLAKE_METADATA
        INSPECT_FLAKE
        TRUE_FLAKE
        FALSE_FLAKE
        NIX_SYSTEMS
        ;
    };
  };
}
