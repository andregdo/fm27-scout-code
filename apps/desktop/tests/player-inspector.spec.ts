import { expect, test } from "playwright/test";

test("player inspector shows other nationalities", async ({ page }) => {
  await page.addInitScript(() => {
    window.localStorage.setItem("fm27scout.language.v1", "en");
  });
  await page.goto("/");
  await page.getByRole("button", { name: "Open last save" }).click();

  await page.getByPlaceholder("Search players, clubs…").fill("Modrić");
  const row = page.locator("tbody tr", { hasText: "Luka Modrić" }).first();
  await expect(row).toBeVisible();
  await row.click();

  const inspector = page.locator("aside").last();
  await expect(inspector).toContainText("Other nationalities:");
  await expect(inspector).toContainText("Croatia");

  // The header carries action buttons; its title must stay on one line and the
  // row must not overflow the aside at a narrower window.
  await page.setViewportSize({ width: 1024, height: 720 });
  const profileTitle = inspector.getByText("Player profile", { exact: true });
  await expect(profileTitle).toBeVisible();
  const titleBox = await profileTitle.boundingBox();
  expect(titleBox).not.toBeNull();
  expect(titleBox!.height).toBeLessThan(20);
  const header = inspector.locator(":scope > div").first();
  const overflow = await header.evaluate(
    (element) => element.scrollWidth - element.clientWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});
