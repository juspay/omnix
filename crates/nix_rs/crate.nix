{ pkgs, ... }:
{
  # Autowiring `crate` so that the tests are run in nix sandbox when `om ci` is used
  autoWire = [ "crate" ];
  crane = {
    args = {
      nativeBuildInputs = with pkgs; [
        nix # Tests need nix cli
      ];
    };
  };
}
