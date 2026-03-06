import { test, expect } from "@microsoft/tui-test";

test.use({
  program: { file: "npm", args: ["run", "tui"] },
});

test("TUI shows Dashboard title", async ({ terminal }) => {
  await expect(terminal.getByText("Dashboard")).toBeVisible();
});

test("TUI shows Cash Flow Projection when opening Cash Flow tab", async ({ terminal }) => {
  await expect(terminal.getByText("Dashboard")).toBeVisible();
  terminal.write("\t");
  await expect(terminal.getByText("Brokers")).toBeVisible();
  terminal.write("\t");
  await expect(terminal.getByText("Unified Positions")).toBeVisible();
  terminal.write("\t");
  await expect(terminal.getByText("Cash Flow Projection")).toBeVisible();
});

test("TUI shows Projection Period label on Cash Flow tab", async ({ terminal }) => {
  await expect(terminal.getByText("Dashboard")).toBeVisible();
  for (let i = 0; i < 3; i++) {
    terminal.write("\t");
  }
  await expect(terminal.getByText("Cash Flow Projection")).toBeVisible();
  await expect(terminal.getByText("Projection Period:")).toBeVisible();
});
