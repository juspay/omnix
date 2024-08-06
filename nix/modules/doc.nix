{ inputs, ... }:

{
  perSystem = { self', pkgs, ... }:
    let
      # The 404.html file has a bizare internal hash.
      siteWithout404File =
        pkgs.runCommand "site-no404" { } ''
          cp -r ${self'.packages.doc} $out
          chmod -R u+w $out
          rm $out/404.html
        '';
    in
    {
      packages.doc = pkgs.callPackage (inputs.self + /doc) { };

      # Check that links are working
      checks.doc-linkCheck =
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

      # This is like `checks.doc-linkCheck`, but also does external link checks
      # (which is something we can't do in Nix due to sandboxing)
      apps.doc-linkCheck.program = pkgs.writeShellApplication {
        name = "doc-linkCheck";
        runtimeInputs = [ pkgs.html-proofer ];
        text = ''
          # Allow Github's line hashes
          htmlproofer \
            --no-check-external-hash \
            ${siteWithout404File}
        '';
      };
    };
}
