{ flake
, pkgs
, lib
, rust-project
, ...
}:

let
  inherit (flake) inputs;
in
{
  autoWire = [ ];
  crane = {
    args = {
      nativeBuildInputs = [
        # nix # Tests need nix cli
      ];
      DEFAULT_FLAKE_SCHEMAS = lib.cleanSourceWith {
        name = "flake-schemas";
        src = flake.inputs.self + /nix/flake-schemas;
      };
      FLAKE_METADATA = lib.cleanSourceWith {
        name = "nix-rs-flake-metadata";
        src = flake.inputs.self + /crates/nix_rs/src/flake/functions/metadata;
      };
      FLAKE_ADDSTRINGCONTEXT = lib.cleanSourceWith {
        name = "nix-rs-flake-addstringcontext";
        src = flake.inputs.self + /crates/nix_rs/src/flake/functions/addstringcontext;
      };
      INSPECT_FLAKE = inputs.inspect;
      TRUE_FLAKE = inputs.true;
      FALSE_FLAKE = inputs.false;
      NIX_SYSTEMS = builtins.toJSON {
        x86_64-linux = lib.cleanSourceWith {
          name = "x86_64-linux";
          src = inputs.nix-systems-x86_64-linux;
        };
        aarch64-linux = lib.cleanSourceWith {
          name = "aarch64-linux";
          src = inputs.nix-systems-aarch64-linux;
        };
        aarch64-darwin = lib.cleanSourceWith {
          name = "aarch64-darwin";
          src = inputs.nix-systems-aarch64-darwin;
        };
        x86_64-darwin = lib.cleanSourceWith {
          name = "x86_64-darwin";
          src = inputs.nix-systems-x86_64-darwin;
        };
      };
    };
  };
}
