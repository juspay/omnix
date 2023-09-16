Next-gen version of <https://github.com/srid/nix-health> in progress. https://github.com/juspay/nix-browser/issues/14

To try it out,

```sh
nix run github:juspay/nix-browser#nix-health
```

nix-health takes an optional flake path or URL to load additional checks from. For eg.,

```sh
nix run github:juspay/nix-browser#nix-health github:nammayatri/nammayatri
```