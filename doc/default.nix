{ stdenv, mdbook, ... }:

stdenv.mkDerivation {
  name = "omnix-mdbook-site";
  src = ./.;

  nativeBuildInputs = [ mdbook ];

  buildPhase = ''
    mdbook build
  '';

  installPhase = ''
    mkdir -p $out
    cp -r book/* $out/
  '';
}
