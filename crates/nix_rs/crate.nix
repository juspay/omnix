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
      nativeBuildInputs = with pkgs; [
        # nix # Tests need nix cli
      ];
      DEFAULT_FLAKE_SCHEMAS = lib.cleanSourceWith {
        name = "flake-schemas";
        src = flake.inputs.self + /nix/flake-schemas;
      };
      INSPECT_FLAKE = inputs.inspect;
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
