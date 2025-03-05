{ lib, ... }:
{
  # Autowiring `crate` so that the tests are run in nix sandbox when `om ci` is used
  autoWire = [ "crate" ];
  crane.args.cargoTestExtraArgs = "-- " + (lib.concatStringsSep " " [
    # requires networking
    "--skip=config::test_get_omconfig_from_remote_flake_with_attr"
  ]);
}
