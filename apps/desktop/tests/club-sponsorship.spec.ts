import { expect, test } from "playwright/test";

test("club inspector shows sponsorship contracts and their status", async ({ page }) => {
  await page.addInitScript(() => {
    window.localStorage.setItem("fm27scout.language.v1", "en");
  });
  await page.goto("/");
  await page.getByRole("button", { name: "Open last save" }).click();
  await page.getByRole("button", { name: "Clubs", exact: true }).click();

  await page.getByText("Casciavit Milano", { exact: true }).first().click();
  const inspector = page.locator("aside").last();
  await inspector.getByTestId("club-tab-sponsorships").click();

  await expect(inspector.getByTestId("sponsorship-active")).toBeVisible();
  await expect(inspector.getByText("Active annual value")).toBeVisible();
  await expect(inspector.getByText("22.00M")).toBeVisible();
  await expect(inspector.getByText("Next expiry")).toBeVisible();
  await expect(inspector.getByText("2027-06-30", { exact: true })).toBeVisible();

  const activeRows = inspector
    .getByTestId("sponsorship-active")
    .getByTestId("sponsorship-row");
  await expect(activeRows).toHaveCount(2);
  await expect(inspector.getByText("Type 1", { exact: true })).toBeVisible();
  await expect(inspector.getByText("Type 4", { exact: true })).toBeVisible();
  await expect(
    inspector.getByText("Falls by 10% on relegation"),
  ).toBeVisible();
  await expect(
    inspector.getByText("Reputation at signing").first(),
  ).toBeVisible();
  await expect(inspector.getByText("Expired and future contracts (2)")).toBeVisible();
  await expect(inspector.locator('[data-status="expired"]')).toHaveCount(1);
  await expect(inspector.locator('[data-status="future"]')).toHaveCount(1);
  await inspector.getByText("Expired and future contracts (2)").click();
  await expect(inspector.getByText("≈ 1.00M", { exact: true })).toBeVisible();
  await expect(
    inspector.getByText("The save stores contract terms only, not sponsor names."),
  ).toBeVisible();
  await expect(inspector.getByText(/Type labels are confirmed/)).toBeVisible();
});

test("club list shows and filters active sponsorship income", async ({ page }) => {
  await page.addInitScript(() => {
    window.localStorage.setItem("fm27scout.language.v1", "en");
  });
  await page.goto("/");
  await page.getByRole("button", { name: "Open last save" }).click();
  await page.getByRole("button", { name: "Clubs", exact: true }).click();

  await expect(
    page.getByRole("button", { name: "Sponsorship", exact: true }),
  ).toBeVisible();
  await expect(page.locator("tbody tr")).toHaveCount(14);

  await page.getByTestId("club-filter-sponsorship").selectOption("true");
  const rows = page.locator("tbody tr");
  await expect(rows).toHaveCount(6);
  await expect(rows.first()).toContainText("Casciavit Milano");
  await expect(rows.first()).toContainText("22.00M");

  await page.getByTestId("club-filter-sponsorship").selectOption("false");
  await expect(rows).toHaveCount(8);
  await expect(page.getByText("Paris Saint-Germain", { exact: true })).toBeVisible();
  await expect(
    page.getByText("Casciavit Milano", { exact: true }),
  ).toHaveCount(0);

  await page.getByTestId("club-filter-sponsorship").selectOption("");
  await expect(rows).toHaveCount(14);
  await page.getByTestId("club-filter-sponsorship-annual-min").fill("17000000");
  await expect(rows).toHaveCount(4);
  await expect(rows.first()).toContainText("Casciavit Milano");
  await expect(page.getByText("Barcelona", { exact: true })).toHaveCount(0);
});

test("analysis centre summarises active sponsorship income", async ({ page }) => {
  await page.addInitScript(() => {
    window.localStorage.setItem("fm27scout.language.v1", "en");
  });
  await page.goto("/");
  await page.getByRole("button", { name: "Open last save" }).click();
  await page.getByRole("button", { name: "Analysis", exact: true }).click();

  const dialog = page.getByRole("dialog");
  await dialog.getByRole("button", { name: "Sponsorship", exact: true }).click();
  const panel = dialog.getByTestId("sponsorship-analysis");
  await expect(panel).toBeVisible();
  await expect(panel.getByText("Clubs with sponsorships")).toBeVisible();
  await expect(panel.getByText("Active contracts")).toBeVisible();
  await expect(
    panel.getByText("Active annual value", { exact: true }).first(),
  ).toBeVisible();
  await expect(panel.getByText("108.00M")).toBeVisible();

  const topClubs = panel.getByRole("row");
  await expect(topClubs.nth(1)).toContainText("Casciavit Milano");
  await expect(topClubs.nth(1)).toContainText("22.00M");
  await expect(panel.getByText("Type 1", { exact: true })).toBeVisible();
  await expect(panel.getByText("Type 4", { exact: true })).toBeVisible();
  await expect(
    panel.getByText("The save stores contract terms only, not sponsor names."),
  ).toBeVisible();

  await topClubs.nth(1).click();
  await expect(dialog).toHaveCount(0);
  const inspector = page.locator("aside").last();
  await expect(inspector.getByTestId("club-tab-sponsorships")).toBeVisible();
  await expect(inspector.getByText("Casciavit Milano").first()).toBeVisible();
});

test("a club without a finance series shows the sponsorship empty state", async ({ page }) => {
  await page.addInitScript(() => {
    window.localStorage.setItem("fm27scout.language.v1", "en");
  });
  await page.goto("/");
  await page.getByRole("button", { name: "Open last save" }).click();
  await page.getByRole("button", { name: "Clubs", exact: true }).click();

  await page.getByText("Paris Saint-Germain", { exact: true }).first().click();
  const inspector = page.locator("aside").last();
  await inspector.getByTestId("club-tab-sponsorships").click();

  await expect(
    inspector.getByText("This save stores no sponsorship contracts for this club."),
  ).toBeVisible();
  await expect(inspector.getByTestId("sponsorship-row")).toHaveCount(0);
});
