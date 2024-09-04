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
        # TODO: Ideally, these params should be moved to upstream module.
        # But do that only as the spec stabilizes.
        om.templates = {
          nix-dev-home = {
            template = inputs.nix-dev-home.templates.default;
            params = [
              {
                name = "username";
                description = "Your username as shown by `whoami`";
                placeholder = "runner";
              }
              # Git
              {
                name = "git-name";
                description = "Your full name for use in Git config";
                placeholder = "John Doe";
              }
              {
                name = "git-email";
                description = "Your email for use in Git config";
                placeholder = "johndoe@example.com";
              }
              # Neovim
              {
                name = "neovim";
                description = "Include Neovim configuration";
                paths = [ "**/neovim.nix" ];
                value = false;
              }
              {
                name = "github-ci";
                description = "Include GitHub Actions workflow configuration";
                paths = [ ".github/**" ];
                value = false;
              }
            ];
          };
          haskell-flake = {
            template = inputs.haskell-flake.templates.example;
            params = [
              {
                name = "package-name";
                description = "Name of the Haskell package";
                placeholder = "example";
              }
            ];
          };
          haskell-template =  {
            template = inputs.haskell-template.templates.default;
            params = [
              {
                name = "author";
                description = "Author name";
                placeholder = "Sridhar Ratnakumar";
              }
              {
                name = "package-name";
                description = "Name of the Haskell package";
                placeholder = "haskell-template";
              }
              {
                name = "vscode";
                description = "Include the VSCode settings folder (./.vscode)";
                paths = [ ".vscode/**" ];
                value = true;
              }
              {
                name = "nix-template";
                description = "Keep the flake template in the project";
                paths = [ "**/template.nix" ];
                value = false;
              }
              {
                name = "github-ci";
                description = "Include GitHub Actions workflow configuration";
                paths = [ ".github/**" ];
                value = true;
              }
            ];
          };
        };
      };
    };
}
