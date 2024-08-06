{ inputs, ... }:

{
  perSystem = { self', pkgs, ... }: {
    packages.doc = pkgs.callPackage (inputs.self + /doc) { };

    # Check that links are working
    checks.doc-linkCheck =
      let
        # The 404.html file has a bizare internal hash.
        siteWithout404File =
          pkgs.runCommand "site-no404" { } ''
            cp -r ${self'.packages.doc} $out
            chmod -R u+w $out
            rm $out/404.html
          '';
      in
      pkgs.runCommand "doc-linkCheck"
        {
          buildInputs = [ pkgs.html-proofer ];
        } ''
        # Ensure that the htmlproofer is using the correct locale
        export LANG=en_US.UTF-8 
        # Run htmlproofer
        htmlproofer --disable-external ${siteWithout404File}
        touch $out
      '';
  };
}
