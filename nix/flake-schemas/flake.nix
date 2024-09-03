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
      omSchema = {
        version = 1;
        doc = ''
          Configuration for `omnix` CLI.
        '';
        inventory = output:
        let
          recurse = prefix: attrs: flake-schemas.lib.mkChildren (builtins.mapAttrs
            (attrName: attrs:
              if (builtins.typeOf attrs) != "set" then
                {
                  value = attrs;
                  what = "omnix config";
                }
              else
                recurse (prefix + attrName + ".") attrs
            )
            attrs);
        in
          recurse "" output;
      };

      nixciSchema = {
        version = 1;
        doc = ''
          Configuration for `nixci`.
        '';
        inventory = output:
        let
          recurse = prefix: attrs: flake-schemas.lib.mkChildren (builtins.mapAttrs
            (attrName: attrs:
              if (builtins.typeOf attrs) != "set" then
                {
                  value = attrs;
                  what = "nixci config";
                }
              else
                recurse (prefix + attrName + ".") attrs
            )
            attrs);
        in
          recurse "" output;
      };

      nixHealthSchema = {
        version = 1;
        doc = ''
          Configuration for `nix-health`.
        '';
        inventory = output:
        let
          recurse = prefix: attrs: flake-schemas.lib.mkChildren (builtins.mapAttrs
            (attrName: attrs:
              if (builtins.typeOf attrs) != "set" then
                {
                  value = attrs;
                  what = "nix-health config";
                }
              else
                recurse (prefix + attrName + ".") attrs
            )
            attrs);
        in
          recurse "" output;
      };
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
