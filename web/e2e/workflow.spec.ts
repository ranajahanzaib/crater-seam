import { expect, test } from "@playwright/test";
test("real dataset walkthrough, findings and downloads", async ({ page }) => {
  await page.goto("/");
  await expect(
    page.getByRole("heading", { name: "Same Moon. Different assumptions." }),
  ).toBeVisible();
  await expect(page.locator(".metrics")).toContainText("384,278");
  await expect(page.locator(".evidence-strip")).toContainText(
    "valid records rejected",
  );
  await page.getByRole("button", { name: "Walkthrough: first crater" }).click();
  await page.getByRole("button", { name: "01-1-00003", exact: true }).click();
  await expect(page.getByLabel("Selected record")).toContainText(
    "has not necessarily moved",
  );
  await expect(page.getByLabel("Selected record")).toContainText("-148.095");
  await page.getByRole("button", { name: "Source & schema" }).click();
  await expect(page.locator(".metadata")).toContainText("BYTE_ARRAY");
  await expect(page.locator(".metadata")).toContainText("-105.383");
  await page.getByLabel("Record ID", { exact: true }).fill("");
  await page
    .getByLabel("Issue", { exact: true })
    .selectOption("LATITUDE_OUT_OF_RANGE");
  await expect(page.locator("tbody").last()).toContainText("excluded");
  await expect(page.locator(".table-area")).toContainText("1 matching");
  await page.locator(".record-link").click();
  await expect(page.getByLabel("Selected record")).toContainText(
    "No validated map position",
  );
  await page.getByRole("button", { name: "Zoom in both maps" }).click();
  await expect(page.locator(".view-controls")).toContainText("1.5×");
  await page.getByRole("button", { name: "Reset view" }).click();
  await expect(page.locator(".view-controls")).toContainText("1.0×");
  await page.getByText("Export audit ↓").click();
  const download = page.waitForEvent("download");
  await page.getByRole("link", { name: "Normalized CSV" }).click();
  expect((await download).suggestedFilename()).toBe("normalized.csv");
});
test("explicit west interpretation and affected filter", async ({ page }) => {
  await page.goto("/");
  await page.getByLabel("A / Simulated importer").selectOption("signed_west");
  await expect(page.locator(".evidence-strip")).toContainText(
    "0 valid records rejected",
  );
  await expect(page.locator(".evidence-strip")).not.toContainText(
    "0 located differently",
  );
  await page.getByLabel("Simulation affected only").check();
  await expect(page.locator(".maps")).not.toHaveAttribute("aria-busy", "true");
  await expect(page.locator(".table-area")).toContainText("matching");
});
test("responsive page and screenshot", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator(".metrics")).toContainText("384,278");
  await expect(page.locator(".maps")).not.toHaveAttribute("aria-busy", "true");
  await page.screenshot({ path: "../docs/screenshot.png", fullPage: true });
  await page.setViewportSize({ width: 390, height: 844 });
  await expect(
    page.getByRole("heading", { name: "Naive interpretation", exact: true }),
  ).toBeVisible();
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth,
    ),
  ).toBe(true);
  await page.screenshot({ path: "../artifacts/mobile.png", fullPage: true });
});
test("selection, pan and zoom stay synchronized; missing texture falls back", async ({
  page,
}) => {
  await page.route("**/lroc_color_2k.jpg", (route) => route.abort());
  await page.goto("/");
  await expect(page.locator(".texture-fallback")).toHaveCount(2);
  await page.getByLabel("A / Simulated importer").selectOption("signed_east");
  await page.getByRole("button", { name: "Walkthrough: first crater" }).click();
  await page.getByRole("button", { name: "01-1-00003", exact: true }).click();
  await expect(page.locator(".target")).toHaveCount(2);
  expect(await page.locator(".target").nth(0).getAttribute("transform")).toBe(
    await page.locator(".target").nth(1).getAttribute("transform"),
  );
  await page.getByRole("button", { name: "Zoom in both maps" }).click();
  await page.getByRole("button", { name: "Zoom in both maps" }).click();
  const canvas = page.locator("canvas").first();
  const box = await canvas.boundingBox();
  if (!box) throw new Error("Missing map canvas");
  await page.mouse.move(box.x + box.width * 0.6, box.y + box.height * 0.6);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width * 0.4, box.y + box.height * 0.4, {
    steps: 5,
  });
  await page.mouse.up();
  expect(await page.locator(".prime").nth(0).getAttribute("x1")).toBe(
    await page.locator(".prime").nth(1).getAttribute("x1"),
  );
  expect(await page.locator(".target").nth(0).getAttribute("transform")).toBe(
    await page.locator(".target").nth(1).getAttribute("transform"),
  );
  await page.getByRole("button", { name: "Reset view" }).click();
  await expect(page.locator(".view-controls")).toContainText("1.0×");
});
