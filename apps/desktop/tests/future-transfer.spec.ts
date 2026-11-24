import { expect, test } from "playwright/test";

test("player table and profile surface a stored future transfer", async ({ page }) => {
  await page.addInitScript(() => {
    window.localStorage.setItem("fm27scout.language.v1", "en");
  });
  await page.goto("/");
  await page.getByRole("button", { name: "Open last save" }).click();

  await page.getByPlaceholder("Search players, clubs…").fill("Modrić");
  const row = page.locator("tbody tr", { hasText: "Luka Modrić" }).first();
  await expect(row).toBeVisible();

  const badge = row.getByTestId("future-transfer-badge");
  await expect(badge).toContainText("Inter");
  await expect(badge).toContainText("2027-07-01");
  await expect(badge).not.toContainText("Napoli");

  await row.click();
  const inspector = page.locator("aside").last();
  const summary = inspector.getByTestId("inspector-future-transfer");
  await expect(summary).toContainText("Future transfer: Inter");
  await expect(summary).toContainText("2027-07-01");
  await expect(summary).not.toContainText("Napoli");

  await inspector.getByRole("button", { name: "Contract", exact: true }).click();
  const card = inspector.getByTestId("inspector-future-transfer-card");
  await expect(card).toContainText("Future transfer");
  await expect(card).toContainText("Inter");
  await expect(card).toContainText("2027-07-01");
  await expect(card).not.toContainText("Napoli");

  // The unagreed future contract stays visible in the chain, but as an offer.
  await expect(inspector.getByText("Napoli")).toBeVisible();
  await expect(inspector.getByText("Offer", { exact: true })).toBeVisible();
});
