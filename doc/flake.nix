{
  inputs = {
    emanote.url = "github:srid/emanote";
    emanote.inputs.emanote-template.follows = "";
    nixpkgs.follows = "emanote/nixpkgs";
    flake-parts.follows = "emanote/flake-parts";
  };

  outputs = inputs@{ self, flake-parts, nixpkgs, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = nixpkgs.lib.systems.flakeExposed;
      imports = [ inputs.emanote.flakeModule ];
      perSystem = { self', pkgs, system, ... }: {
        emanote = {
          sites."default" = {
            layers = [{ path = ./.; pathString = "."; }];
            baseUrl = "/"; # Keep URLs the same as mdBook
            prettyUrls = true;
          };
        };


        # Check that links are working
        checks.linkCheck =
          let site = self'.packages.default; in
          pkgs.runCommand "linkCheck"
            {
              buildInputs = [ pkgs.html-proofer ];
            } ''
            # Ensure that the htmlproofer is using the correct locale
            export LANG=en_US.UTF-8
            # Run htmlproofer on the generated site
            htmlproofer --disable-external ${site}
            echo success > $out
          '';

        apps = {
          # This is like `checks.linkCheck`, but also does external link checks
          # (which is something we can't do in Nix due to sandboxing)
          linkCheck.program = pkgs.writeShellApplication {
            name = "linkCheck";
            runtimeInputs = [ pkgs.html-proofer ];
            text = ''
              set -x
              # Build the site first
              SITE=$(nix build --print-out-paths --no-link)
              # Allow Github's line hashes
              htmlproofer \
                --no-check-external-hash \
                "$SITE"
            '';
          };
        };
      };
    };
}
