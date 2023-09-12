# Playwright end-to-end tests

We use [Playwright](https://playwright.dev/dotnet/) to test our application.

- All e2e test are nixified, simply run `nix run .#e2e-playwright-test` from project root (there are `just` targets as well)
- The nix shell creates a `node_modules` symlink which in turn provide IDE support in VSCode for editing `tests/*`
