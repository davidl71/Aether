# IB Box Spread Web Interface

React + Vite SPA that mirrors the terminal UI: live status banner, multi-tab dashboard (symbols, current/historic positions, orders, alerts), box-spread scenario explorer, and quick-action controls.

## Architecture

- **Data flow**: `useSnapshot` polls `/data/snapshot.json` (or `VITE_API_URL`) via `SnapshotClient`, matching the TUI `Snapshot` schema. `useBoxSpreadData` hydrates scenario tables from `/data/box_spread_sample.json`. Both hooks bubble errors into the UI.
- **State layout**: `App` maintains active tab + detail modal state. Keyboard shortcuts (`B`, `Shift+S`) trigger the same mock combo actions as the TUI.
- **UI composition**: Header status banner, scenario summary/table, pill-based tabs, reusable data grids with sparkline overlays, timeline + alert feeds, and modal detail sheets.
- **Styling**: Dark trading desk palette via `src/styles/app.css`, responsive down to tablet width. SVG-based sparklines emulate TUI candle strips.
- **Testing**: Vitest + Testing Library cover the dashboard render path with mocked snapshot/scenario responses.

## Extending Toward Production

1. Swap `/data/snapshot.json` for a REST/WS gateway (see `SnapshotClient`), emitting the same JSON produced for the TUI.
2. Replace the combo action placeholders (`handleBuyCombo`, `handleSellCombo`) with REST/Socket calls into the order manager.
3. Expand alert/order feeds with infinite scroll or server-sent events.
4. Integrate charting (candles, heatmaps) once QuestDB / ORATS payloads are exposed.

## Scripts

- `npm run dev` – start Vite dev server.
- `npm run build` – produce optimized bundle in `dist/`.
- `npm run preview` – preview the build locally.
- `npm run lint` – ESLint flat config (React, TypeScript).
- `npm run test` – Vitest run mode with jsdom.

Bootstrap with:

```bash
bash agents/web/scripts/setup.sh   # npm install
bash agents/web/scripts/run-tests.sh
npm run dev
```

Set `VITE_API_URL=https://host/api/snapshot` to hit a live backend. The default static JSON under `public/data/` keeps the SPA functional offline.
