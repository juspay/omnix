{ pkgs
, lib
, ...
}:
{
  autoWire = [ ];
  crane = {
    args = {
      nativeBuildInputs = with pkgs; lib.optionals stdenv.isDarwin [
        libiconv
        pkg-config
      ];
      buildInputs = lib.optionals pkgs.stdenv.isDarwin
        lib.optionals
        pkgs.stdenv.isLinux [
        pkgs.openssl
      ];
      # Disable tests due to sandboxing issues; we run them on CI
      # instead.
      doCheck = false;
    };
  };
}
