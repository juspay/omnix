{
  nixConfig = {
    extra-substituters = "https://cache.garnix.io";
    extra-trusted-public-keys = "cache.garnix.io:CTFPyKSLcx5RMJKfLo5EEPUObbA78b0YQ2DTCJXqr9g=";
  };

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
          pkgs.runCommand "linkCheck"
            {
              buildInputs = [ pkgs.html-proofer ];
            } ''
            # Ensure that the htmlproofer is using the correct locale
            export LANG=en_US.UTF-8
            # Run htmlproofer on the generated site
            htmlproofer --disable-external ${inputs.emanote.lib.${system}.mkEmanoteDerivation self'.emanote.sites.default}
            touch $out
          '';

        apps = {
          # This is like `checks.doc-linkCheck`, but also does external link checks
          # (which is something we can't do in Nix due to sandboxing)
          linkCheck.program = pkgs.writeShellApplication {
            name = "linkCheck";
            runtimeInputs = [ pkgs.html-proofer ];
            text = ''
              set -x
              # Allow Github's line hashes
              htmlproofer \
                --no-check-external-hash \
                ${inputs.emanote.lib.${system}.mkEmanoteDerivation self'.emanote.sites.default}
            '';
          };
        };
      };
    };
}
