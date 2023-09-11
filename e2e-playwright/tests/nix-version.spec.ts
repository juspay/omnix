import { test, expect } from "@playwright/test";

test('check nix version', async ({ page }) => {
  await page.goto('/info');
  const nixVersion = await page.locator(":text('Nix Version') + div").textContent();
  await expect(nixVersion).toBeTruthy();
});
