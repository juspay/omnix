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
      };
    };
}
