# WIP: nix-browser

[Project Doc](https://docs.google.com/document/d/1tcpufxKFdaqmyKL0qpK174zexY14LV69C44459h8VmQ/edit#heading=h.5x0d5h95i329)

## Getting Started

1. [Install Nix](https://zero-to-flakes.com/install)
1. [Setup `direnv`](https://zero-to-flakes.com/direnv)
1. Clone this repo, `cd` to it, and run `direnv allow`.

This will automatically activate the nix develop shell. Open VSCode and install recommended extensions, making sure direnv activates in VSCode as well.

## Running locally

In nix shell,

```
just watch
```

## Nix workflows

Inside the nix develop shell (activated by direnv) you can use any of the `cargo` or `rustc` commands, as well as [`just`](https://just.systems/) workflows. Nix specific commands can also be used to work with the project:

```sh
# Full nix build
nix build

# Build and run
nix run
```
