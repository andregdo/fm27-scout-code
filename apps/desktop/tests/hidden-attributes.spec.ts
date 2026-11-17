import { expect, test } from "playwright/test";

test("hidden attributes are always visible in the profile and opt-in in the table", async ({ page }) => {
  await page.addInitScript(() => {
    window.localStorage.setItem("fm27scout.language.v1", "en");
  });
  await page.goto("/");
  await page.getByRole("button", { name: "Open last save" }).click();

  // No hidden-attribute toggle is needed any more.
  await expect(page.getByRole("button", { name: "Hidden attributes" })).toHaveCount(0);

  await page.getByPlaceholder("Search players, clubs…").fill("Modrić");
  const row = page.locator("tbody tr", { hasText: "Luka Modrić" }).first();
  await expect(row).toBeVisible();

  // The Hidden column is off by default.
  await expect(row).not.toContainText("Pro 20");

  // It can still be shown from the column menu.
  await page.getByRole("button", { name: "Columns" }).click();
  await page.getByRole("button", { name: "Hidden" }).click();
  await expect(row).toContainText("Pro 20");

  await row.click();

  const inspector = page.locator("aside").last();
  // There is no separate Hidden tab; the information lives in Attributes.
  await expect(inspector.getByRole("button", { name: "Hidden", exact: true })).toHaveCount(0);
  await expect(inspector).toContainText("Hidden attributes");
  await expect(inspector).toContainText("Consistency");
  await expect(inspector).toContainText("Personality");
  await expect(inspector).toContainText("Professionalism");
});
