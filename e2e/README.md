# Playwright end-to-end tests

We use [Playwright](https://playwright.dev/dotnet/) to test our application.

- All e2e test are nixified, simply run `nix run .#e2e-playwright-test` from project root
- A `package.json` exists for better IDE support (autocompletion, hover docs etc)
