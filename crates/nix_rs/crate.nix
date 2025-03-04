{
  autoWire = [ ];
  crane = {
    args = {
      nativeBuildInputs = [
        # nix # Tests need nix cli
      ];
    };
  };
}
