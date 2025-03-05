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
        pkgs.nix
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

      cargoTestExtraArgs = "-- " + (lib.concatStringsSep " " [
        # requires networking
        "--skip=command::ci::build_flake_output"
        "--skip=command::ci::test_haskell_multi_nix"
        "--skip=command::ci::test_haskell_multi_nix_all_dependencies"
        "--skip=command::ci::test_services_flake"
        "--skip=command::show::om_show_nixos_configurations"
        "--skip=command::show::om_show_remote"

        # required health checks, like "Flakes Enabled" and "Max Jobs" fails in sandbox
        "--skip=command::health::om_health"

        # error: creating directory '/nix/var/nix/profiles': Permission denied
        # TODO: elaborate on the error
        "--skip=command::init::om_init"

        # FIXME: This should pass in the sandbox
        # > ❄\u{fe0f}  nix -j auto eval --json --override-input flake ../.. --override-input flake-schemas /nix/store/0npxvlidr0w1b99lrvcmd2wd46y2kibh-flake-schemas --override-input systems /nix/store/g1pv78p6lk92hjzr7syrcihvj4rx1fnz-source --no-write-lock-file \'/nix/store/qm8q4kylp1jy68k34c2bskpamxgyc9ix-source#contents.excludingOutputPaths\' --quiet --quiet\u{fe0f}
        # > error: experimental Nix feature \'nix-command\' is disabled; use \'--extra-experimental-features nix-command\' to override
        # > Error: Unable to fetch flake
        "--skip=command::show::om_show_local"
      ]);
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
