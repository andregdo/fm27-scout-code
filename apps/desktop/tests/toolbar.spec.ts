import { expect, test } from "playwright/test";

test("always shows full information without a data mode selector", async ({ page }) => {
  await page.addInitScript(() => {
    window.localStorage.setItem("fm27scout.language.v1", "en");
    // Older builds persisted Moneyball; it must be ignored now.
    window.localStorage.setItem("fm27scout.dataMode.v1", "moneyball");
  });
  await page.goto("/");
  await page.getByRole("button", { name: "Open last save" }).click();

  await expect(page.getByText("Fair play")).toHaveCount(0);
  await expect(page.getByText("Moneyball")).toHaveCount(0);
  // Moneyball hid CA/PA; Full keeps them visible.
  await expect(page.getByRole("columnheader", { name: "CA", exact: true })).toBeVisible();
  await expect(page.getByRole("columnheader", { name: "PA", exact: true })).toBeVisible();
});
