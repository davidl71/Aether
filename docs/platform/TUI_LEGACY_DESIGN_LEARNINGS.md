# TUI Legacy Design Learnings

**Purpose:** Features and design patterns from legacy TUI docs that the current Rust/Ratatui TUI can learn from.  
**Sources:** `docs/research/architecture/TUI_DESIGN.md`, `TUI_SCENARIO_EXPLORER_DESIGN.md`, `TUI_MULTISCREEN_RESEARCH.md`, `docs/TICKER_TUI_ANALYSIS.md`, `docs/platform/TUI_RUST_READ_PATH_AUDIT.md`, `docs/CACHE_AND_INTERVAL_DEFAULTS.md`.  
**Current TUI:** `agents/backend/services/tui_service` (Ratatui + crossterm).  
**TUI data path:** NATS only (api.*); no direct database or backend HTTP.

---

## 1. Layout & information density

| Pattern | Legacy doc | Current state | Learning |
|--------|------------|----------------|----------|
| **Header line with mode + strategy + account** | TUI_DESIGN: "Mode: DRY-RUN \| Strategy: RUNNING \| Account: DU123456" | Status bar has mode, strategy, source; account could be explicit | Add account id/label in header when snapshot has it. |
| **Subsystem status pills** | TUI_DESIGN: "TWS: OK \| ORATS: Enabled \| Portal: OK \| QuestDB: OK" | Single "source" + "stale" indicator | Consider multiple backend/connector pills (NATS, backend health, IB if present). |
| **Persistent alerts strip** | TUI_DESIGN: Bottom "Alerts" row always visible | Alerts are a full tab only | Optional: small alert strip or last-N line in footer so alerts visible without switching tab. |
| **top/htop familiarity** | TUI_DESIGN: "Mimic top/htop layout" | Tab bar + content area is similar | Keep Tab/1–5 navigation; consider htop-style F-keys in hint bar for power users. |

---

## 2. Shortcuts & actions

| Feature | Legacy doc | Current state | Learning |
|---------|------------|----------------|----------|
| **S / T (strategy start/stop)** | TUI_DESIGN | ✅ Implemented (NATS) | Keep; already matches. |
| **K – cancel all open strategy orders** | TUI_DESIGN | ❌ Not present | Add NATS handler for "cancel all" and key `K` when scope approved. |
| **D – toggle dry-run mode** | TUI_DESIGN | ❌ Not present | Add if backend supports dry-run toggle via NATS/API; else document as future. |
| **Enter – detail popup for selected row** | TUI_DESIGN, Scenario Explorer | ❌ No row popup | Add "Enter = detail popup" for Positions/Orders (e.g. order legs, position P&amp;L). Reuse or wire `events.rs` EventRouter when adding popups. |
| **B – Buy combo (prefilled)** | TUI_DESIGN | ❌ Not present | Major feature: prefilled combo order from selected spread/symbol; needs backend support. Defer to product scope. |
| **Shift+S – Sell combo** | TUI_DESIGN | ❌ Not present | Same as B; defer. |
| **Tab / Shift+Tab cycle tabs** | TUI_DESIGN | ✅ Tab / ← → and 1–5 | Already there. |
| **Arrow keys scroll lists** | TUI_DESIGN, Scenario Explorer | Partial (Logs; Orders filter) | Add arrow-key scroll for Positions and Alerts tables. |

---

## 3. Tabs and content

| Tab / content | Legacy doc | Current state | Learning |
|---------------|------------|----------------|----------|
| **Dashboard** | Symbols, profitability, subsystem status | ✅ Dashboard with symbols + sparklines | Consider adding profitability summary line (e.g. net P&amp;L, buying power if backend exposes). |
| **Current Positions** | ROI, maker/taker counts, rebate estimates | ✅ Positions table | Add ROI column; maker/taker/rebate if data available from backend. |
| **Historic Positions** | QuestDB + strategy history | ❌ No historic tab | Add "Historic" or "Closed" tab when backend/snapshot provides closed positions. |
| **Orders** | Timeline of live/past | ✅ Orders + filter | Keep; consider sort by time descending by default. |
| **Alerts** | Scrollable feed | ✅ Alerts tab | Add arrow-key scroll; optional "last N in footer" for visibility. |
| **Scenarios** | Scenario Explorer doc | ❌ Missing | New tab or Dashboard section: scenario summary (total, avg APR, probable count, max APR) + sortable scenario table (symbol, expiry, width, debit, profit, ROI, APR, fill prob). Data: backend API or file-based like snapshot. |

---

## 4. Color and semantics

| Rule | Legacy doc | Current state | Learning |
|------|------------|----------------|----------|
| Headers / healthy | Cyan/green | Similar | Keep consistent. |
| ROI / liquidity / buying power | Green positive, red negative | Partial | Apply consistently to P&amp;L and balance fields. |
| Maker/taker | Cyan maker, magenta taker | N/A | Use when displaying maker/taker counts. |
| Alerts | Info blue, warning yellow, error red | Likely in Alerts tab | Standardize by severity. |
| Config-driven palette / monochrome fallback | TUI_DESIGN "Config-driven palette" | Not implemented | Optional: theme or monochrome mode from config for accessibility. |

---

## 5. Scenario explorer (from TUI_SCENARIO_EXPLORER_DESIGN.md)

- **Summary block:** Total scenarios, average APR, probable count (fill_prob &gt; 0), max APR with example (e.g. "SPX 2025-12-19 5000/5100").
- **Table:** Symbol, Expiration, Strike Width, Net Debit, Profit, ROI %, APR %, Fill Probability; sortable columns; arrow keys + Enter for details.
- **Data:** Prefer same source as web (e.g. `/api/v1/scenarios` or file-based refresh). Recommendation in doc: file-based for consistency with snapshot.
- **Filters:** By symbol, expiration; sort by APR desc; "probable only" toggle.
- **External reference:** [boxtrades.com](https://www.boxtrades.com/) shows SPX/ES box spread yields by expiration; use for UX alignment and expiry bucket labels. See `docs/platform/BOXTRADES_REFERENCE.md`.

### 5b. Yield tab: symbol selection and benchmark curve (historical design)

- **Symbol selection:** Watchlist (from `strategy.symbols` / config); ←/→ on Yield tab to change symbol. Box curve and benchmark are shown per selected symbol when wired.
- **Box spread curve:** Data from `api.finance_rates.build_curve` (NATS). Columns: Symbol, Expiry, DTE, Bucket (expiry_buckets), Rate %. Reference: boxtrades.com.
- **Benchmark yield curve:** Treasury and SOFR from `api.finance_rates.benchmarks` (FRED when `FRED_API_KEY` set). Overlay/comparison per [NATS_API.md](NATS_API.md) and task “Overlay Treasury yield curve on box spread yield curve comparison”. TUI shows placeholder table (Tenor, Rate %, Source) until NATS response is wired.

---

## 6. Multiscreen / split panes (from TUI_MULTISCREEN_RESEARCH.md)

- **Idea:** Show multiple tabs at once (e.g. Dashboard | Positions, or 2×2) instead of single-tab view.
- **References:** Zellij, lazygit, k9s, btop, lazydocker.
- **Ratatui:** Use layout splits (`Layout::default().direction(Direction::Horizontal).split()` etc.) to put two or four content areas side by side; optional resize with mouse or keys.
- **Phases:** (1) Dashboard + Positions side-by-side; (2) 2×2 grid; (3) configurable layout.
- **Benefit:** Less tab switching; better use of large terminals.

---

## 7. Patterns from TICKER_TUI_ANALYSIS (bubbletea/Go)

- **Config:** YAML/watchlists (we have shared JSON); custom color schemes and display options → map to our config-driven palette idea.
- **Position tracking:** Cost basis, multiple lots, auto value calc → align with Positions tab and any future cost-basis columns.
- **Data refresh:** Polling vs push; we use NATS push + snapshot TTL → keep; document refresh semantics in one place.
- **Summary statistics:** Ticker-style summary line → same as "profitability summary" on Dashboard.

---

## 8. Implementation patterns

| Pattern | Legacy doc | Learning |
|---------|------------|----------|
| **Modular views** | TUI_DESIGN "Modular views to keep panels reusable" | Keep `render_dashboard`, `render_positions`, etc.; extract shared table/block helpers if duplication grows. |
| **Background data** | TUI_DESIGN "Background goroutines poll REST or WebSocket" | We use NATS subscription + snapshot; optional REST fallback already considered. |
| **Event routing** | events.rs TODO | When adding popups, filters, or multi-handler tabs, wire `EventRouter` / `EventPriority` or adopt a small ratatui pattern so key handling stays consistent. |
| **Detail popup** | TUI_DESIGN + Scenario Explorer "Enter to view details" | Use a centered overlay (e.g. `centered_rect` like help overlay); Esc to close; content from selected row. |

---

## 9. Quick reference: doc → feature map

| Document | Main takeaways |
|----------|----------------|
| **TUI_DESIGN.md** | top/htop layout, header (mode/strategy/account), status pills, S/T/K/D/B/Shift+S, Enter popup, color rules, modular views, config palette. |
| **TUI_SCENARIO_EXPLORER_DESIGN.md** | Scenarios tab/section: summary stats + sortable table, filters, keyboard nav, data source options. |
| **TUI_MULTISCREEN_RESEARCH.md** | Split panes (2 or 4), resizable, focus management; reference apps (lazygit, k9s, btop). |
| **TICKER_TUI_ANALYSIS.md** | Config and display options, position/cost basis, summary stats, polling vs push. |
| **TUI_RUST_READ_PATH_AUDIT.md** | Which reads are Rust vs legacy; no revival of Python TUI. |
| **CACHE_AND_INTERVAL_DEFAULTS.md** | Legacy TUI intervals as reference only; Rust TUI uses snapshot TTL / NATS. |

---

## 10. Suggested priority for current TUI

1. **Low effort, high value:** Arrow-key scroll for Positions and Alerts; explicit account in header if present in snapshot; ROI column on Positions if backend provides it.
2. **Medium:** Enter = detail popup for selected row (Orders/Positions); optional K = cancel all strategy orders (with backend support).
3. **Larger:** Scenario explorer (summary + table); optional split-pane layout; D and B/Shift+S when backend supports them.

Use this file as a checklist when adding features or refactoring the TUI.

---

## 11. Design patterns from completed exarp TUI tasks

Below are patterns and decisions inferred from **Done** Todo2 tasks tagged or related to TUI (55 tasks). Use these as established patterns when adding or refactoring TUI features.

### 11.1 Patterns now in the codebase (reuse and extend)

| Pattern | Task(s) | Where in code | Learning |
|--------|---------|----------------|----------|
| **Help overlay with ?** | T-57 | `app.show_help`, `ui::render_help_overlay`, `centered_rect` | Modal overlays: one key to open (?), any key to close; center over content. Reuse for future detail popups. |
| **Sparkline trend column** | T-1773357423912362000 | `app.roi_history`, `ui::render_dashboard` (Trend column), `roi_history_to_sparkline_data` | Per-row sparklines from a deque of values; use Ratatui built-in Sparkline widget (T-1773509396768831000: sufficient). |
| **Interactive order filter** | T-1773357423945485000 | `app.order_filter`, `/` to focus, type to filter, Esc to clear; filter by symbol/status/side | In-tab filter state; show "Filter: &lt;text&gt;" when active; filter list before rendering. Same pattern can apply to Alerts or Positions. |
| **Strategy S/T via NATS** | (recent NATS-only work) | `strategy_cmd_tx`, `run_strategy_commands`, hint bar result (✓ or error) | Commands from TUI → NATS request/reply → result back to TUI. Skip S/T on Orders tab so `/` filter can use s/t. |
| **Config validation hint** | T-1773264125158584000, parity | `validate_config_hint()`, `config_warning` in status bar on load/reload | Validate NATS_URL, BACKEND_ID (and optional fields); show short hint only, no block. |
| **Stale snapshot indication** | T-1773264125139437000 | `TuiSnapshot::is_stale(ttl_secs)`, status bar color + "Updated Xs ago", [STALE] when over TTL | Single TTL from config; yellow when stale, green when fresh. |
| **Non-interactive fallback** | T-1773265204305175000 | Startup: require TTY; message if not | Fail fast with clear message when not a TTY; no headless UI. |
| **Scrolling/persistence for Logs** | T-177326519821070000 | Logs tab: tui-logger widget, scroll position, level filter (+/-) | Dedicated scroll/level state per tab when content is long-lived. |
| **Shared config as canonical** | T-114, T-160, T-1773221160047995000 | `config.rs` load from shared JSON, env overrides, hot-reload | One discovery path (IB_BOX_SPREAD_CONFIG / config paths); env overrides; reload on interval. |
| **NATS-only data path** | T-1773514704051364000 (REST fallback removed) | main.rs: NATS subscription only; no REST snapshot | Single source: NATS. No dual REST+NATS for snapshot. |
| **Tab + number keys** | T-56, T-57, T-58 | Tab/BackTab/←/→; 1–5 jump to Dash/Pos/Orders/Alerts/Logs | Consistent: Tab cycle, 1–5 direct jump. Preserve when adding tabs. |
| **Event routing deferred** | T-1773357423959019000, T-1773514704906631000 | events.rs: EventRouter/EventPayload present but unused; comment says "wire when adding popups, filters" | Use when adding multiple overlays or per-component handlers; keep single key-handler in app until then. |

### 11.2 Architectural decisions (follow these)

| Decision | Task(s) | Learning |
|----------|---------|----------|
| **Rust TUI as default** | T-1773227845816309000, T-1773221160047995000 | Rust TUI is the canonical terminal UI; Python/Textual retired. New terminal features go in tui_service. |
| **api_base_url / NATS as canonical** | T-1773221160047995000, T-1773221168380194000 | Single canonical way for TUI to reach backend (NATS); reduce duplicate health/routing paths. |
| **Shared read models in Rust** | T-1773174764827468000, T-1773174378092953000, T-1773172060167170000, T-1773172054316351000 | Unified positions, bank normalization, relationship inference live in Rust; TUI consumes snapshot that already reflects them. |
| **Snapshot contract aligned** | T-1773172068617486000 | Web and TUI share same snapshot shape; keep one contract (e.g. proto or shared JSON). |
| **Config validation = hint only** | Parity doc | Don’t block startup; show warning in status bar. Full schema validation optional. |
| **Multiscreen research done, split layout optional** | T-53, T-54, T-55, T-56, T-203 | Multiscreen patterns documented (TUI_MULTISCREEN_RESEARCH.md); 2-pane split was implemented then possibly simplified. Use layout splits when we want multiple panes visible; not required for MVP. |
| **Bubble Tea as reference only** | T-1773180671128897000 | Evaluated as future Go option; no Go TUI in this repo. Patterns (e.g. from TICKER_TUI_ANALYSIS) still useful. |

### 11.3 Deferred or optional (revisit when scope allows)

| Topic | Task(s) | Note |
|-------|---------|------|
| **Popup row details + cancel confirmation** | T-1773357423930509000 | Enter = detail popup and cancel confirmation were in scope; implement using same overlay pattern as help. |
| **Box spread scenario explorer** | T-14 | Scenario summary + table; see §3 and §5 in this doc. |
| **WebSocket real-time** | T-15 | Superseded by NATS push for TUI; no need to add WebSocket for TUI. |
| **Symbol jump / add (G key)** | T-58, T-168 | Quick key to add or jump to symbol; tests (T-168) done. Re-add if product wants it. |
| **TUI test coverage** | T-201 | Add tests for TUI modules when touching them; prefer unit tests for app logic and key handling. |
| **Feature parity script** | T-1773512007882708000 | check_feature_parity.sh exists; run for doc summary. |

### 11.4 Task → pattern quick map

| Task ID | Pattern or decision |
|---------|----------------------|
| T-57 | Help modal + ? key |
| T-56 | Tab/split layout (split optional now) |
| T-114, T-160 | Shared config for TUI |
| T-1773357423912362000 | Sparkline trend column |
| T-1773357423945485000 | Order filter (/ and text) |
| T-1773357423930509000 | Popup details + cancel (deferred) |
| T-1773264125139437000 | Surface stale snapshot (yes; TTL + color) |
| T-1773264125158584000 | Validate shared-config behavior |
| T-1773514704051364000 | NATS-only; no REST fallback |
| T-1773514704906631000 | Event routing deferred until needed |
| T-1773509396768831000 | Ratatui Sparkline sufficient |

Use §11 when adding a new TUI feature: prefer patterns from §11.1, respect §11.2, and consider §11.3 for backlog.
