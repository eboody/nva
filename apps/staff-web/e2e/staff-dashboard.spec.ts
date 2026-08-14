import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

const seriousOrCritical = ["serious", "critical"];

test("staff can navigate the rendered tool portfolio with semantic controls", async ({ page }) => {
  await page.goto("/");

  await expect(page.getByRole("heading", { level: 1, name: "Owned Operations Platform" })).toBeVisible();
  await expect(page.getByRole("navigation", { name: "Demo organization" })).toBeVisible();
  await expect(page.getByRole("main")).toBeVisible();

  const bookingTriage = page.getByRole("button", { name: "Inspect Intake / Booking Triage lineage" });
  await bookingTriage.click();
  await expect(bookingTriage).toHaveAttribute("aria-pressed", "true");
  await expect(page.getByText("Intake / Booking Triage", { exact: true }).first()).toBeVisible();
});

test("dashboard has no serious or critical automated accessibility violations", async ({ page }) => {
  await page.goto("/");
  const results = await new AxeBuilder({ page }).analyze();
  const blocking = results.violations.filter((violation) => seriousOrCritical.includes(violation.impact ?? ""));
  expect(blocking).toEqual([]);
});

test("API failure remains visible and fail-safe", async ({ page }) => {
  await page.route("**/api/local-demo/**", async (route) => {
    await route.fulfill({
      status: 503,
      contentType: "application/json",
      body: JSON.stringify({
        error: { code: "local_demo_unavailable", message: "Demo backend unavailable" },
        live_side_effects_allowed: false
      })
    });
  });
  await page.goto("/");
  await page.getByRole("button", { name: "Run information lifespan" }).first().click();

  const liveRegion = page.locator(".stage-machine-live-region");
  await expect(liveRegion).toContainText("request failed safely; no live side effects attempted");
  await expect(liveRegion).toContainText("Demo backend unavailable");
});

test("responsive dashboard does not overflow the viewport", async ({ page }) => {
  await page.goto("/");
  await page.addStyleTag({
    content: "body, button { font-family: monospace !important; }"
  });
  await expect(page.getByRole("heading", { level: 1 })).toBeVisible();

  const dimensions = await page.evaluate(() => ({
    viewportWidth: window.innerWidth,
    documentWidth: document.documentElement.scrollWidth
  }));
  expect(dimensions.documentWidth).toBeLessThanOrEqual(dimensions.viewportWidth);
});

test("primary controls are keyboard reachable", async ({ page }) => {
  await page.goto("/");
  const run = page.getByRole("button", { name: "Run information lifespan" }).first();
  await run.focus();
  await expect(run).toBeFocused();
  await page.keyboard.press("Tab");
  await expect(page.getByRole("button", { name: "Replay / reset" })).toBeFocused();
});
