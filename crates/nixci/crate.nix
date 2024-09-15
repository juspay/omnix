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
  autoWire = [ "doc" "clippy" ];
  crane = {
    args = {
      nativeBuildInputs = with pkgs; with pkgs.apple_sdk_frameworks; lib.optionals stdenv.isDarwin [
        Security
        SystemConfiguration
      ] ++ [
        libiconv
        pkg-config
      ];
      buildInputs = lib.optionals pkgs.stdenv.isDarwin
        (
          with pkgs.apple_sdk_frameworks; [
            IOKit
            CoreFoundation
          ]
        ) ++ lib.optionals pkgs.stdenv.isLinux [
        pkgs.openssl
      ];
      # Disable tests due to sandboxing issues; we run them on CI
      # instead.
      doCheck = false;
      inherit (rust-project.crates."nix_rs".crane.args)
        DEFAULT_FLAKE_SCHEMAS
        INSPECT_FLAKE
        ;
      DEVOUR_FLAKE = inputs.devour-flake;
      NIX_SYSTEMS = builtins.toJSON {
        x86_64-linux = inputs.nix-systems-x86_64-linux;
        aarch64-linux = inputs.nix-systems-aarch64-linux;
        x86_64-darwin = inputs.nix-systems-x86_64-darwin;
        aarch64-darwin = inputs.nix-systems-aarch64-darwin;
      };
      OMNIX_SOURCE = rust-project.src;
    };
  };
}
