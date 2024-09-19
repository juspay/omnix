{
  inputs = {
    flake-schemas.url = "github:DeterminateSystems/flake-schemas";
  };
  outputs = { flake-schemas, ... }:
    let
      appsSchema = {
        version = 1;
        doc = ''
          The `apps` output provides commands available via `nix run`.
        '';
        inventory = output:
          flake-schemas.lib.mkChildren (builtins.mapAttrs
            (system: apps:
              let
                forSystems = [ system ];
              in
              {
                inherit forSystems;
                children =
                  builtins.mapAttrs
                    (appName: app: {
                      inherit forSystems;
                      evalChecks.isValidApp =
                        app ? type
                        && app.type == "app"
                        && app ? program
                        && builtins.isString app.program;
                      what = "app";
                      shortDescription = app.meta.description or "";
                    })
                    apps;
              })
            output);
      };
      nixosConfigurationsSchema = {
        version = 1;
        doc = ''
          The `nixosConfigurations` flake output defines [NixOS system configurations](https://nixos.org/manual/nixos/stable/#ch-configuration).
        '';
        inventory = output: flake-schemas.lib.mkChildren (builtins.mapAttrs
          (configName: machine:
            {
              what = "NixOS configuration";
              derivation = machine.config.system.build.toplevel;
              forSystems = [ machine.pkgs.stdenv.system ];
              # Force evaluate this derivation on all systems. See: https://github.com/juspay/omnix/pull/277#discussion_r1760164052
              evalOnAllSystems = true;
            })
          output);
      };
      homeConfigurationsSchema = {
        version = 1;
        doc = ''
          The `homeConfigurations` flake output defines [Home Manager configurations](https://github.com/nix-community/home-manager).
        '';
        inventory = output: flake-schemas.lib.mkChildren (builtins.mapAttrs
          (configName: this:
            {
              what = "Home Manager configuration";
              derivation = this.activationPackage;
              forSystems = [ this.activationPackage.system ];
              evalOnAllSystems = true;
            })
          output);
      };
      darwinConfigurationsSchema = {
        version = 1;
        doc = ''
          The `darwinConfigurations` flake output defines [nix-darwin configurations](https://github.com/LnL7/nix-darwin).
        '';
        inventory = output: flake-schemas.lib.mkChildren (builtins.mapAttrs
          (configName: this:
            {
              what = "nix-darwin configuration";
              derivation = this.system;
              forSystems = [ this.system.system ];
              evalOnAllSystems = true;
            })
          output);
      };
    in
    {
      schemas = flake-schemas.schemas // {
        # Until upstream is merged: https://github.com/DeterminateSystems/flake-schemas/pull/31
        apps = appsSchema;
        nixosConfigurations = nixosConfigurationsSchema;
        homeConfigurations = homeConfigurationsSchema;
        darwinConfigurations = darwinConfigurationsSchema;
      };
    };
}
