{ flake
, rust-project
, pkgs
, lib
, ...
}:

let
  inherit (flake) inputs;
  inherit (pkgs) stdenv pkgsStatic;
in
{
  autoWire = lib.optionals
    (lib.elem pkgs.system [ "x86_64-linux" "aarch64-darwin" ])
    [ "doc" "clippy" ];
  crane = {
    args = {
      nativeBuildInputs = with pkgs.apple_sdk_frameworks; lib.optionals stdenv.isDarwin [
        Security
        SystemConfiguration
      ] ++ [
        # Packages from `pkgsStatic` require cross-compilation support for the target platform,
        # which is not yet available for `x86_64-apple-darwin` in nixpkgs. Upon trying to evaluate
        # a static package for `x86_64-apple-darwin`, you may see an error like:
        #
        # > error: don't yet have a `targetPackages.darwin.LibsystemCross for x86_64-apple-darwin`
        (if (stdenv.isDarwin && stdenv.isAarch64) then pkgsStatic.libiconv else pkgs.libiconv)
        pkgs.pkg-config
      ];
      buildInputs = lib.optionals pkgs.stdenv.isDarwin
        (
          with pkgs.apple_sdk_frameworks; [
            IOKit
            CoreFoundation
          ]
        ) ++ lib.optionals pkgs.stdenv.isLinux [
        pkgsStatic.openssl
      ];

      inherit (rust-project.crates."nix_rs".crane.args)
        DEFAULT_FLAKE_SCHEMAS
        INSPECT_FLAKE
        NIX_SYSTEMS
        ;
      inherit (rust-project.crates."omnix-ci".crane.args)
        DEVOUR_FLAKE
        ;
      inherit (rust-project.crates."omnix-init".crane.args)
        OM_INIT_REGISTRY
        ;
      inherit (rust-project.crates."omnix-health".crane.args)
        CACHIX_BIN
        ;

      # To avoid unnecessary rebuilds, start from cleaned source, and then add the Nix files necessary to `nix run` it. Finally, add any files required by the Rust build.
      OMNIX_SOURCE = lib.cleanSourceWith {
        src = inputs.self;
        filter = path: type:
          rust-project.crane-lib.filterCargoSources path type
            || lib.hasSuffix ".nix" path
            || lib.hasSuffix "flake.lock" path
            || lib.hasSuffix "registry.json" path
        ;
      };

      # Disable tests due to sandboxing issues; we run them on CI
      # instead.
      doCheck = false;
      meta = {
        description = "Command-line interface for Omnix";
        mainProgram = "om";
      };
      CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";

      hardeningDisable = [ "fortify" ]; # https://github.com/NixOS/nixpkgs/issues/18995#issuecomment-249748307
    } //
    lib.optionalAttrs (stdenv.isLinux && stdenv.isAarch64) {
      CARGO_BUILD_TARGET = "aarch64-unknown-linux-musl";
    } //
    lib.optionalAttrs (stdenv.isLinux && stdenv.isx86_64) {
      CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
    };
  };
}
