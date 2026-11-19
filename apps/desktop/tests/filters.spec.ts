import { expect, test } from "playwright/test";

test("gender filter offers only men and women", async ({ page }) => {
  await page.addInitScript(() => {
    window.localStorage.setItem("fm27scout.language.v1", "en");
  });
  await page.goto("/");
  await page.getByRole("button", { name: "Open last save" }).click();

  await expect(page.getByTestId("gender-all")).toContainText("320");
  await expect(page.getByTestId("gender-men")).toContainText("256");
  await expect(page.getByTestId("gender-women")).toContainText("64");
  await expect(page.getByTestId("gender-unknown")).toHaveCount(0);

  await page.getByTestId("gender-women").click();
  await expect(page.getByText("64 players", { exact: true })).toBeVisible();
});

test("reputation column shows current reputation and filters the list", async ({ page }) => {
  await page.addInitScript(() => {
    window.localStorage.setItem("fm27scout.language.v1", "en");
  });
  await page.goto("/");
  await page.getByRole("button", { name: "Open last save" }).click();

  await expect(page.getByRole("columnheader", { name: "Reputation" })).toBeVisible();
  await page.getByRole("columnheader", { name: "Reputation" }).click();
  const top = page.locator("tbody tr").first();
  await expect(top).toContainText("Luka Modrić");
  await expect(top).toContainText("9,500");

  await page.getByTestId("reputation-range-min").fill("9000");
  await expect(page.getByText("1 players", { exact: true })).toBeVisible();
  await expect(page.locator("tbody tr")).toHaveCount(1);
  await expect(page.locator("tbody tr").first()).toContainText("Luka Modrić");

  await page.getByTestId("reputation-range-min").fill("9600");
  await expect(page.getByText("No matching players")).toBeVisible();
});
