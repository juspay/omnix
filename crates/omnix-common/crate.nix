{ flake
, rust-project
, pkgs
, lib
, ...
}:

{
  autoWire = [ "doc" "clippy" ];
  crane = {
    args = {
      inherit (rust-project.crates."nix_rs".crane.args)
        DEVOUR_FLAKE
        DEFAULT_FLAKE_SCHEMAS
        NIX_FLAKE_SCHEMAS_BIN
        ;
    };
  };
}
