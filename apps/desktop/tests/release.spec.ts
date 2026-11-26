import { expect, test } from "playwright/test";

test("stages a release and saves it with pending changes", async ({ page }) => {
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

  await page.getByRole("button", { name: "Release", exact: true }).click();
  const dialog = page.getByRole("dialog");
  await expect(dialog).toContainText("Release player");
  await expect(dialog).toContainText("Currently at Tottenham Hotspur");
  await expect(dialog).toContainText("Squad 21 → 20");
  await expect(dialog).toContainText(
    "The contract record, wage, squad number and listing bits are kept.",
  );
  await expect(dialog).not.toContainText(/[一-鿿]/);

  await dialog.getByRole("button", { name: "Add to pending changes" }).click();
  await expect(dialog).not.toBeVisible();
  await expect(page.getByText("Release staged")).toBeVisible();

  await page.getByRole("button", { name: /Pending/ }).click();
  const pending = page.getByRole("dialog");
  await expect(pending).toContainText("Pending changes");
  await expect(pending).toContainText("Releases");
  await expect(pending).toContainText("Tommaso Nielsen");
  await expect(pending).toContainText("Tottenham Hotspur → Free agent");
  await expect(pending).toContainText("Squad 21 → 20");

  await pending.getByRole("button", { name: "Save all to a copy…" }).click();
  await expect(pending).toContainText("Pending changes written");
  await expect(pending).toContainText("BB KING - Casciavit - edited.fm");
});
