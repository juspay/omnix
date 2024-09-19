{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";

    # Pull templates from external flake-parts modules
    # The pull happens in CI periodically.
    haskell-flake.url = "github:srid/haskell-flake";
    haskell-template.url = "github:srid/haskell-template";
    nix-dev-home.url = "github:juspay/nix-dev-home";
    rust-nix-template.url = "github:srid/rust-nix-template";
  };
  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;

      perSystem = { pkgs, ... }: {
        packages.hello = pkgs.hello; # Dummy output for `om ci`
      };

      flake =
        let
          inherit (inputs.nixpkgs) lib;
          # Accumulate om.templates from all inputs
          inputsTemplates =
            let
              templateSets = lib.mapAttrsToList
                (name: input:
                  if name == "self"
                  then { }
                  else lib.attrByPath [ "om" "templates" ] { } input
                )
                inputs;
            in
            builtins.foldl' (acc: set: acc // set) { } templateSets;
        in
        {
          om.templates = inputsTemplates;
        };
    };
}
