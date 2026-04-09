# TUI implementation backlog (planning)

**Last updated:** 2026-04-05  
**Canonical context:** [`TUI_ARCHITECTURE.md`](./TUI_ARCHITECTURE.md), [`TUI_PANE_MODEL.md`](./TUI_PANE_MODEL.md)  
**Scope:** design notes and checklists only — no large refactors implied.

| Task ID | Topic |
|---------|--------|
| T-1774817606420467000 | ScrollableTableState: API + ownership |
| T-1774817606441825000 | Toasts: queue, policy, concurrency |
| T-1774817606463474000 | Numeric columns: measurement helper API |
| T-1774817606507094000 | Research: docs index survey (this file §5) |
| T-1774817606353005000 | Pane model: regression checklist |
| T-1774807919706789000 | Shared layout / hint helpers (post–workspace split) |

---

## 1. ScrollableTableState — API + ownership (T-1774817606420467000)

**Today:** `agents/backend/services/tui_service/src/scrollable_table.rs` — `ScrollableTableState { selected, scroll }` with getters, `move_up` / `move_down` / `adjust_scroll` / `clamp_to_len` / `reset` / `shift_selected` / `shift_scroll`. Free helpers: `clamp_index`, `centered_viewport_start`.

**Ownership:** `App` owns one state struct per logical table (`positions_table`, `orders_table`, `dashboard_table`, …). Call sites clamp after data changes; viewport helpers used from `positions`, `orders`, `loans` UIs.

**Design notes (incremental, no breaking churn):**

- Keep **single owner** on `App` unless a sub-widget needs temporary state (prefer `&mut` borrow from `App`).
- Document **when** to call `clamp_to_len` vs `adjust_scroll` in module rustdoc (already stated; extend with “after resize” if needed).
- Optional later: `new(selected, scroll)` / `with_selected` for tests only; avoid public setters that bypass invariants.
- Stragglers: any remaining ad hoc `usize` scroll on table-like UIs should migrate per pane model (see [`TUI_PANE_MODEL.md`](./TUI_PANE_MODEL.md) §5).

---

## 2. Toasts — queue, policy, concurrency (T-1774817606441825000)

**Implementation:** `agents/backend/services/tui_service/src/ui/feedback.rs` (`ToastManager`), driven from `App::push_toast` / `app_updates` / tick cleanup.

| Aspect | Current behavior |
|--------|------------------|
| Storage | `VecDeque<Toast>` + monotonic `id` |
| Visible cap | `MAX_VISIBLE_TOASTS` (3), severity-aware selection for overlay |
| TTL | `DEFAULT_TOAST_DURATION` / `ERROR_TOAST_DURATION` per `ToastLevel` |
| Queue cap | `MAX_QUEUED_TOASTS` (32) — drops oldest via `max_history` trim |
| Dedupe | Same **normalized** message among **active** toasts refreshes existing TTL |
| Concurrency | Single-threaded `App` mutation on main loop; no cross-thread `ToastManager` access |

**Planning:** If async producers appear, define a single `mpsc` or `AppEvent` path into `push_toast` — do not share `ToastManager` across threads. Document “one writer” invariant in `feedback.rs` when touching this.

---

## 3. Numeric columns — measurement helper API (T-1774817606463474000)

**Goal:** Right-align numeric table columns without copy-pasted `width` math (see [`TUI_ARCHITECTURE.md`](./TUI_ARCHITECTURE.md) Phase 3 table polish).

**Proposed small API (sketch):**

- `fn display_width_cells(s: &str) -> usize` — Unicode-aware display width if needed, else `s.chars().count()` initially.
- `fn max_numeric_width(samples: impl Iterator<Item = impl AsRef<str>>) -> usize` — max over formatted strings.
- `fn format_numeric_column(w: usize, value: &str) -> String` — pad left to `w` (or `Span` builder for ratatui).

**Constraints:** Keep in `ui/` helper module; no new dependencies unless `unicode-width` (or similar) is already in the workspace — verify `Cargo.toml` before adding.

---

## 4. Pane model — regression checklist (T-1774817606353005000)

Canonical **checkbox list:** [`TUI_PANE_MODEL.md`](./TUI_PANE_MODEL.md) §7. Run after focus/router/workspace changes.

Summary (see §7 for the full list):

- **Layer stack:** Overlays capture keys before tab body; `App::input_mode()` precedence unchanged.
- **Tabs / workspaces / Settings / scroll / mouse / narrow terminal / toasts** — as in §7.

---

## 5. Docs / README survey — TUI-related entries (T-1774817606507094000)

Inventory of first-class **TUI** docs under `docs/` (excluding `docs/archive/` and deep `research/` unless listed).

| Doc | Role |
|-----|------|
| [`TUI_ARCHITECTURE.md`](./TUI_ARCHITECTURE.md) | Main loop, file map, planned phases |
| [`TUI_PANE_MODEL.md`](./TUI_PANE_MODEL.md) | Focus, workspaces, scroll ownership |
| [`TUI_INPUT_DECISION.md`](./TUI_INPUT_DECISION.md) | Input policy |
| [`TUI_RATATUI_INTERACT.md`](./TUI_RATATUI_INTERACT.md) | ratatui-interact pilots |
| [`TUI_PANTRY_WORKFLOW.md`](./TUI_PANTRY_WORKFLOW.md) | Widget dev loop |
| [`TUI_UX_BENCHMARKS.md`](./TUI_UX_BENCHMARKS.md) | UX benchmarks |
| [`TUI_AUDIT_2026_03_24.md`](./TUI_AUDIT_2026_03_24.md) | Audit snapshot |
| [`SETTINGS_SHORT_TERMINAL_QA.md`](./SETTINGS_SHORT_TERMINAL_QA.md) | Narrow Settings QA |
| [`research/TUI_IMPLEMENTATION_PATTERNS.md`](./research/TUI_IMPLEMENTATION_PATTERNS.md) | Pattern notes |

**Index:** [`README.md`](./README.md) lists `TUI_ARCHITECTURE.md` and related architecture rows; add this backlog there only if the team wants it in the global index (optional).

---

## 6. Shared layout / hint helpers — extraction map (T-1774807919706789000)

**Done / partial:** `ui/chrome_layout.rs` — vertical chrome split (`split_vertical_chrome`, height constants). `workspace.rs` — workspace membership and focus cycling.

**Duplication candidates (consolidate behind small modules, not big-bang):**

| Location | Item | Suggested home |
|----------|------|----------------|
| `ui/mod.rs`, `discoverability.rs`, `ui/tree_panel.rs` | `centered_rect(percent_x, percent_y, r)` | `ui/layout_helpers.rs` or extend `chrome_layout.rs` |
| `ui/mod.rs` | Composed workspace horizontal splits + tab body routing | Keep orchestration in `mod.rs`; extract **pure** `Rect → [Rect]` helpers if reused |
| `discoverability.rs` | `context_hints_for` + hint strings | Keep; align with `render_hint_bar` when bindings change ([`TUI_ARCHITECTURE.md`](./TUI_ARCHITECTURE.md) Help vs hints) |
| `settings.rs` | `settings_layout_*` / min-width thresholds | Already tab-local; only extract if second consumer appears |

**Non-goals:** Moving all render logic into one file; changing `InputMode` routing in the same pass as layout nits.

---

## References

- Code: `agents/backend/services/tui_service/src/`
- Pane verification: [`TUI_PANE_MODEL.md`](./TUI_PANE_MODEL.md) §7
