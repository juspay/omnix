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
  };
  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;
      flake = {
        templates = {
          nix-dev-home = inputs.nix-dev-home.templates.default;
          haskell-flake = inputs.haskell-flake.templates.example;
          haskell-template = inputs.haskell-template.templates.default;
        };
        # TODO: Ideally, these params should be moved to upstream module.
        # But do that only as the spec stabilizes.
        om.templates = {
          nix-dev-home = {
            tags = [ "home-manager" "juspay" "development" ];
            params = [
              {
                name = "Username";
                help = "Your username as shown by by $USER";
                default = "runner";
                required = true;
                files = [
                  "flake.nix"
                ];
              }
              {
                name = "Full Name";
                help = "Your full name for use in Git config";
                default = "John Doe";
                required = true;
                files = [
                  "home/default.nix"
                ];
              }
              {
                name = "Email";
                help = "Your email for use in Git config";
                default = "johndoe@example.com";
                required = true;
                files = [
                  "home/default.nix"
                ];
              }
              {
                name = "GitHub Actions CI";
                help = "Include GitHub Actions workflow configuration";
                default = false;
                files = [
                  ".github"
                ];
              }
            ];
          };
          haskell-flake = {
            tags = [ "haskell" "haskell-flake" ];
            params = [
              {
                name = "Package Name";
                help = "Name of the Haskell package";
                default = "example";
                required = false;
                files = [
                  "example.cabal"
                  "flake.nix"
                ];
              }
            ];
          };
          haskell-template = {
            tags = [ "haskell" "haskell-flake" "relude" "just" ];
            params = [
              {
                name = "Package Name";
                help = "Name of the Haskell package";
                default = "haskell-template";
                required = false;
                files = [
                  "haskell-template.cabal"
                  "flake.nix"
                  "justfile"
                ];
              }
              {
                name = "Author";
                help = "Author name";
                default = "Sridhar Ratnakumar";
                required = false;
                files = [
                  "haskell-template.cabal"
                  "LICENSE"
                ];
              }
              {
                name = "VSCode support";
                help = "Include the VSCode settings folder (./.vscode)";
                default = true;
                files = [
                  ".vscode"
                ];
              }
              {
                name = "GitHub Actions CI";
                help = "Include GitHub Actions workflow configuration";
                default = true;
                files = [
                  ".github"
                ];
              }
            ];
          };
        };
      };
    };
}
