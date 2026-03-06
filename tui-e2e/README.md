# TUI E2E Tests

End-to-end tests for the Python Textual TUI using [@microsoft/tui-test](https://github.com/microsoft/tui-test).

## Requirements

- **Node.js** 18.x or 20.x (`>=16.6.0 <21.0.0`; Node 21+ is not yet supported by @microsoft/tui-test’s native pty dependency). Use `nvm use 20` or similar if needed.
- **uv** and Python TUI deps (see repo root); TUI runs with mock provider.

## Setup

From repo root or `tui-e2e/`:

```bash
cd tui-e2e && npm install
```

## Run tests

From `tui-e2e/`:

```bash
npm test
```

Or from repo root:

```bash
cd tui-e2e && npm test
```

With trace (for debugging):

```bash
cd tui-e2e && npm run test:trace
```

Traces are written to `tui-e2e/tui-traces/` and can be replayed with the tui-test `show-trace` command.

## What is tested

- **Dashboard** – TUI starts and shows the Dashboard title
- **Tabs** – Tab to Cash Flow and confirm "Cash Flow Projection" and "Projection Period:" are visible
- **Projection period** – Label visible on Cash Flow tab

The TUI is started with `TUI_BACKEND=mock` so no live backend (IB, REST) is required.

## Configuration

Edit `tui-test.config.ts` to change retries, enable trace by default, etc.

## Adding tests

Add new `*.spec.ts` files in `tui-e2e/` or extend `tui.spec.ts`. Use the same pattern:

```ts
import { test, expect } from "@microsoft/tui-test";

test.use({
  program: { file: "npm", args: ["run", "tui"] },
});

test("your scenario", async ({ terminal }) => {
  await expect(terminal.getByText("Some text")).toBeVisible();
  terminal.write("\t");  // tab
  await expect(terminal).toMatchSnapshot();
});
```
