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
        INSPECT_FLAKE
        NIX_SYSTEMS
        ;
    };
  };
}
