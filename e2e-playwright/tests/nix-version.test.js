// @ts-check
const { test, expect } = require('@playwright/test');

test('check nix version', async ({ page, request }) => {
  await page.goto('/info');
  const nixVersion = await page.locator(":text('Nix Version') + div").textContent();
  await expect(nixVersion).toBeTruthy();
});
