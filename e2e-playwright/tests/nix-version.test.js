// @ts-check
const { test, expect } = require('@playwright/test');

test('check nix version', async ({ page, request }) => {
  await page.goto('/info');
  const nixVersion = await page.locator(":text('Nix Version') + div").textContent();

  const apiResponse = await request.get(`/api/data/nix-info`);
  expect(apiResponse.ok()).toBeTruthy();

  const nixInfo = await apiResponse.json();
  const nixVer = nixInfo.nix_version;
  const nixVerStr = `${nixVer.major}.${nixVer.minor}.${nixVer.patch}`;

  await expect(nixVersion).toBe(nixVerStr);
});
