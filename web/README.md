# IB Box Spread Web Interface

React + Vite SPA that mirrors the terminal UI: live status banner, multi-tab dashboard (symbols, current/historic positions, orders, alerts), box-spread scenario explorer, and quick-action controls.

**Now includes Progressive Web App (PWA) support** - install on mobile and desktop devices for offline access and native app-like experience.

## Architecture

- **Data flow**: `useSnapshot` polls `/data/snapshot.json` (or `VITE_API_URL`) via `SnapshotClient`, matching the TUI `Snapshot` schema. `useBoxSpreadData` hydrates scenario tables from `/data/box_spread_sample.json`. Both hooks bubble errors into the UI.
- **State layout**: `App` maintains active tab + detail modal state. Keyboard shortcuts (`B`, `Shift+S`) trigger the same mock combo actions as the TUI.
- **UI composition**: Header status banner, scenario summary/table, pill-based tabs, reusable data grids with sparkline overlays, timeline + alert feeds, and modal detail sheets.
- **Styling**: Dark trading desk palette via `src/styles/app.css`, responsive down to tablet width. SVG-based sparklines emulate TUI candle strips.
- **Testing**: Vitest + Testing Library cover the dashboard render path with mocked snapshot/scenario responses.
- **PWA**: Service worker caching, offline support, installable on mobile/desktop, automatic updates.

## Progressive Web App (PWA)

The app is fully configured as a PWA with:

- **Offline Support**: Service worker caches assets and data for offline access
- **Installable**: Can be installed on iOS, Android, and desktop browsers
- **Auto Updates**: Service worker automatically updates when new versions are available
- **Smart Caching**:
  - Images: Cache-first (30 days)
  - API calls: Network-first with 5-minute cache
  - Data files: Stale-while-revalidate (1 day)

### Installing the PWA

**Desktop (Chrome/Edge):**
1. Visit the app in your browser
2. Click the install icon in the address bar
3. Or use the browser menu: "Install IB Box Spread Dashboard"

**Mobile (iOS Safari):**
1. Open the app in Safari
2. Tap the Share button
3. Select "Add to Home Screen"

**Mobile (Android Chrome):**
1. Open the app in Chrome
2. Tap the menu (three dots)
3. Select "Install app" or "Add to Home screen"

### Generating Icons

Icons are required for PWA installation. Generate them from a source image:

```bash
# Generate icons from a 512x512 PNG source image
./scripts/generate-icons.sh path/to/your-icon.png

# Or create a placeholder icon automatically
./scripts/generate-icons.sh
```

The script requires ImageMagick:
- macOS: `brew install imagemagick`
- Linux: `apt-get install imagemagick` or `yum install ImageMagick`

Icons will be generated in `public/icons/` with all required sizes (72x72 through 512x512).

## Extending Toward Production

1. Swap `/data/snapshot.json` for a REST/WS gateway (see `SnapshotClient`), emitting the same JSON produced for the TUI.
2. Replace the combo action placeholders (`handleBuyCombo`, `handleSellCombo`) with REST/Socket calls into the order manager.
3. Expand alert/order feeds with infinite scroll or server-sent events.
4. Integrate charting (candles, heatmaps) once QuestDB / ORATS payloads are exposed.
5. Add push notifications for alerts and order updates (PWA supports this).

## Scripts

- `npm run dev` – start Vite dev server (PWA enabled in dev mode).
- `npm run build` – produce optimized bundle in `dist/` with service worker.
- `npm run preview` – preview the build locally.
- `npm run lint` – ESLint flat config (React, TypeScript).
- `npm run test` – Vitest run mode with jsdom.

Bootstrap with:

```bash
bash agents/web/scripts/setup.sh   # npm install
./scripts/generate-icons.sh         # Generate PWA icons
bash agents/web/scripts/run-tests.sh
npm run dev
```

Set `VITE_API_URL=https://host/api/snapshot` to hit a live backend. The default static JSON under `public/data/` keeps the SPA functional offline, and the service worker caches it for offline access.

## Feature Parity with TUI

This web app is designed to mirror the Terminal User Interface (TUI) functionality. See [Feature Tracking](../docs/FEATURE_TRACKING.md) for:
- Complete feature comparison between TUI and Web App
- Feature status (implemented, partial, missing)
- Implementation locations
- Feature gaps and priorities

Run `./scripts/check_feature_parity.sh` from the repo root to verify feature parity.
