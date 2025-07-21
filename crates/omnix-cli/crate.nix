{ pkgs
, lib
, ...
}:

let
  inherit (pkgs) stdenv pkgsStatic;
in
{
  autoWire = [ ];
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

      # Enable tests to run via Nix  
      doCheck = true;

      # Disable sandbox for tests that require network access
      __noChroot = true;
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
