{ flake
, pkgs
, lib
, rust-project
, ...
}:

{
  autoWire = [ ];
  crane.args = {
    buildInputs = lib.optionals pkgs.stdenv.isDarwin (
      with pkgs.apple_sdk_frameworks; [
        IOKit
      ]
    );
    inherit (rust-project.crates."nix_rs".crane.args)
      DEFAULT_FLAKE_SCHEMAS
      INSPECT_FLAKE
      NIX_SYSTEMS
      ;
    OM_INIT_REGISTRY =
      lib.cleanSourceWith {
        name = "om-init-registry";
        src = flake.inputs.self + /crates/omnix-init/registry;
      };
  };
}
