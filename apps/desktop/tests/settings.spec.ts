import { expect, test } from "playwright/test";

test("settings reports and cleans the local cache", async ({ page }) => {
  await page.addInitScript(() => {
    window.localStorage.setItem("fm27scout.language.v1", "en");
  });
  await page.goto("/");
  await page.getByRole("button", { name: "Open last save" }).click();
  await page.getByRole("button", { name: "Settings" }).click();

  const dialog = page.getByRole("dialog");
  await expect(dialog.getByText("Cache database · 24.0 MB")).toBeVisible();
  await expect(
    dialog.getByText("1 cached saves · 0 stale · 0 orphaned rows"),
  ).toBeVisible();

  await dialog.getByRole("button", { name: "Clean up" }).click();
  await expect(page.getByText("Cache already clean")).toBeVisible();
});
