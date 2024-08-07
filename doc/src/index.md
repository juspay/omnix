# Introduction

**Omnix** aims to supplement the [Nix](https://nixos.asia/en/nix) CLI to improve developer experience.

<p style="text-align: center; float: right">
<img src="favicon.svg" alt="Omnix Logo" width="32px" />
</p>

> [!WARNING] 
> 🚧 omnix is in active development. View [the Github repo](https://github.com/juspay/omnix) for ongoing progress.


## Install

To install Omnix, you first need [Nix installed](https://nixos.asia/en/install),[^static] before running the following:

```sh
# Install omnix
nix --accept-flake-config profile install github:juspay/omnix

# Make sure that the `om` command works
om --help
```

## Next Steps

Checkout the [CLI](om/index.md) commands available.

[^static]: We also plan to provide a static binary. See [#207](https://github.com/juspay/omnix/issues/207)