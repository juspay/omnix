{ stdenv, mdbook, mdbook-alerts, ... }:

stdenv.mkDerivation {
  name = "omnix-mdbook-site";
  src = ./.;

  nativeBuildInputs = [ mdbook mdbook-alerts ];

  buildPhase = ''
    mdbook build
  '';

  installPhase = ''
    mkdir -p $out
    cp -r book/* $out/

    # Remove the default, since we don't have our own
    # We will use favicon.svg
    rm $out/favicon.png
  '';
}
