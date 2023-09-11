# WIP: nix-browser

## Getting Started

1. [Install Nix](https://zero-to-flakes.com/install)
1. [Setup `direnv`](https://zero-to-flakes.com/direnv)
1. Clone this repo, `cd` to it, and run `direnv allow`.

This will automatically activate the nix develop shell. Open VSCode and install recommended extensions, making sure direnv activates in VSCode as well.

> [!NOTE] 
> If you would like to **learn the tools & technology** involved in this project, follow along this README noting the places where the emoji 🎓 is used.

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

## Contributing

- When you are done with your changes, run `just fmt` to **autoformat** the source tree; the CI checks for this.
- Add tests if relevant, and run them:
    - Run `just test` to run the **unit tests**.
    - Run `just e2e` (requires `just watch` to be running) or `just e2e-release` to run the **end-to-end tests**
- Add documentation wherever useful. To preview the **docs**, run `just doc`.

## Frontend tech

### Rust wasm

We use [Leptos](https://leptos.dev/). With sufficient knowledge of Rust, you can 🎓 read the [Leptos Book](https://leptos-rs.github.io/leptos/) to get familiar with reactive frontend programming in Rust.

### Styling

We use [Tailwind](https://tailwindcss.com/) for styling; 🎓 familiarize yourself with it! Tailwind enables developers not familiar with design to create reasonably good looking sites. You should also 🎓 get familiar with CSS flexboxes (see [Flexbox Froggy](https://flexboxfroggy.com/)).

#### Color palette

See `tailwind.config.js` for colour aliases we use throughout the app. Instead of, say, `text-pink-500` we use `text-primary-500` ("primary" is more semantic than "pink").

## Crates

We publish the following crates from this repo:

| Crate Link | Description | 
|------------|-------------|
| https://crates.io/crates/nix_rs | Rust interface to the Nix command line |
| https://crates.io/crates/nix_health | Nix health check library and executable |
