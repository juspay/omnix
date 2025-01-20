{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    flake = { };
    include-inputs = { };
  };
  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      perSystem = { pkgs, lib, ... }:
        let
          include-inputs = inputs.include-inputs.value;
        in
        {
          packages = {
            all = pkgs.writeText "nix_rs-metadata.json" (builtins.toJSON {
              # *All* nested inputs are flattened into a single list of inputs.
              inputs = if !include-inputs then null else
              let
                inputsFor = visited: prefix: f:
                  let
                    here = builtins.unsafeDiscardStringContext "${f.outPath}";
                  in
                  # Keep track of visited nodes to workaround a nasty Nix design wart that leads to infinite recursion otherwise.
                    # https://github.com/NixOS/nix/issues/7807
                    # https://github.com/juspay/omnix/pull/389
                  lib.optionals (!lib.hasAttr here visited)
                    (lib.concatLists (lib.mapAttrsToList
                      (k: v: [{ name = "${prefix}__${k}"; path = v.outPath; }] ++
                        (lib.optionals (lib.hasAttr "inputs" v))
                          (inputsFor (visited // { "${here}" = true; }) "${prefix}/${k}" v))
                      f.inputs));
              in
              inputsFor { } "flake" inputs.flake;
              flake = inputs.flake.outPath;
            });
          };
        };
    };
}
