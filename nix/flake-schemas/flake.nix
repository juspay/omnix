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
      mkTraverseSchema = name: doc: {
        version = 1;
        inherit doc;
        inventory = output:
          let
            traverse = prefix: attrs: flake-schemas.lib.mkChildren (builtins.mapAttrs
              (attrName: value:
                if (builtins.typeOf value) != "set" then
                  {
                    value = value;
                    what = "${name} config";
                  }
                else
                  traverse (prefix + attrName + ".") value
              )
              attrs);
          in
          traverse "" output;
      };
      omSchema = mkTraverseSchema "omnix" ''
        Configuration for `omnix`.
      '';

      nixciSchema = mkTraverseSchema "nixci" ''
        Configuration for `nixci`.
      '';

      nixHealthSchema = mkTraverseSchema "nix-health" ''
        Configuration for `nix-health`.
      '';
    in
    {
      schemas = flake-schemas.schemas // {
        # Until upstream is merged: https://github.com/DeterminateSystems/flake-schemas/pull/31
        apps = appsSchema;

        # Custom schemas
        om = omSchema;
        nixci = nixciSchema;
        nix-health = nixHealthSchema;
      };
    };
}
