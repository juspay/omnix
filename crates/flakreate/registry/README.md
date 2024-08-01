## `om init` template registry

The default template registry for `om init`.

## Cache

>[!TIP]
> Omnix will use the cache if it exits, to sidestep slow evaluation due to lockfile explosion.

The flake output is cached in `flake.nix.json`.  To create or update the cache, run:

```sh
nix run .#cache
```

The cache must be kept in sync with `flake.{nix,lock}`. This is checked in CI.
