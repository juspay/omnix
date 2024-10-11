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
  autoWire = lib.optionals
    (lib.elem pkgs.system [ "x86_64-linux" "aarch64-darwin" ])
    [ "doc" "clippy" ];
  crane = {
    args = {
      nativeBuildInputs = with pkgs; [
        nix # Tests need nix cli
      ];
      DEFAULT_FLAKE_SCHEMAS = lib.cleanSourceWith {
        name = "flake-schemas";
        src = flake.inputs.self + /nix/flake-schemas;
      };
      INSPECT_FLAKE = inputs.inspect;
      NIX_SYSTEMS = builtins.toJSON {
        x86_64-linux = inputs.nix-systems-x86_64-linux;
        aarch64-linux = inputs.nix-systems-aarch64-linux;
        x86_64-darwin = inputs.nix-systems-x86_64-darwin;
        aarch64-darwin = inputs.nix-systems-aarch64-darwin;
      };
    };
  };
}
