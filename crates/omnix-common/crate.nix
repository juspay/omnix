{ flake
, rust-project
, pkgs
, lib
, ...
}:

{
  autoWire = lib.optionals
    (lib.elem pkgs.system [ "x86_64-linux" "aarch64-darwin" ])
    [ "doc" "clippy" ];
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
