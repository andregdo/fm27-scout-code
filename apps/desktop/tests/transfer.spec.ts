import { expect, test } from "playwright/test";

test("moves a player to another club through the transfer dialog", async ({ page }) => {
  await page.addInitScript(() => {
    window.localStorage.setItem("fm27scout.language.v1", "en");
  });
  await page.goto("/");
  await page.getByRole("button", { name: "Open last save" }).click();

  // The mock reuses names; the Tottenham row is the only Tommaso Nielsen there.
  await page.getByPlaceholder("Search players, clubs…").fill("Tommaso Nielsen");
  const row = page
    .locator("tbody tr", { hasText: "Tommaso Nielsen" })
    .filter({ hasText: "Tottenham Hotspur" })
    .first();
  await expect(row).toBeVisible();
  await row.click();

  await page.getByRole("button", { name: "Transfer", exact: true }).click();
  const dialog = page.getByRole("dialog");
  await expect(dialog).toContainText("Move player");

  await dialog.getByPlaceholder("Search clubs…").fill("Juventus");
  const destination = dialog.getByRole("button", { name: /Juventus/ });
  await expect(destination).toBeVisible();
  await destination.click();

  await expect(dialog).toContainText("Squad 21 → 20");
  await expect(dialog).toContainText("Squad 22 → 23");
  await expect(dialog).toContainText("Contract restarts on 2026-07-13");
  await expect(dialog).toContainText(
    "Squad number 13 cleared; the new club assigns one after loading.",
  );

  await dialog.getByRole("button", { name: "Write transferred save…" }).click();
  await expect(dialog).toContainText("Transferred save written");
  await expect(dialog).toContainText("BB KING - Casciavit - transferred.fm");
});
