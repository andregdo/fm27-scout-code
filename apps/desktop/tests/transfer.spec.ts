import { expect, test } from "playwright/test";

test("stages a transfer and saves it with pending changes", async ({ page }) => {
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
  // Save nation names are Chinese in this save; the English UI must translate them.
  await expect(destination).toContainText("Portugal");
  await expect(destination).not.toContainText(/[一-鿿]/);
  await destination.click();

  await expect(dialog).toContainText("Squad 21 → 20");
  await expect(dialog).toContainText("Squad 22 → 23");
  await expect(dialog).toContainText("Contract restarts on 2026-07-13");
  await expect(dialog).toContainText(
    "Squad number 13 cleared; the new club assigns one after loading.",
  );
  await expect(dialog).not.toContainText(/[一-鿿]/);

  await dialog.getByRole("button", { name: "Add to pending changes" }).click();
  await expect(dialog).not.toBeVisible();
  await expect(page.getByText("Transfer staged")).toBeVisible();

  await page.getByRole("button", { name: /Pending/ }).click();
  const pending = page.getByRole("dialog");
  await expect(pending).toContainText("Pending changes");
  await expect(pending).toContainText("Tommaso Nielsen");
  await expect(pending).toContainText("Tottenham Hotspur → Juventus");
  await expect(pending).toContainText("#13 cleared");

  await pending.getByRole("button", { name: "Save all to a copy…" }).click();
  await expect(pending).toContainText("Pending changes written");
  await expect(pending).toContainText("BB KING - Casciavit - edited.fm");
});
