# TUI pane & focus model (target architecture)

**Status:** design note — implements backlog **T-1774817606352659000** and informs **T-1774463349681383000** (EPIC).  
**Code anchors:** `workspace.rs`, `pane.rs`, `app.rs` (`InputMode`, `secondary_focus`, `visible_workspace`), `ui/mod.rs`, `scrollable_table.rs`, `mode.rs`.

---

## 1. Goals

- **One mental model** for operators: *where am I* (tab / workspace), *what owns keys* (input mode + pane), *what scrolls* (table state).
- **Predictable routing:** overlays and forms capture keys first; base navigation sees leftovers.
- **Composable workspaces** without duplicating min-size rules or focus cycles (already centralized in `WorkspaceSpec`).

---

## 2. Layer stack (render order, bottom → top)

| Layer | Responsibility | Primary types |
|-------|----------------|---------------|
| **Global chrome** | Tab bar, main rect, hint bar, status bar | `ui/mod.rs`, `chrome_layout` constraints |
| **Workspace shell** | Market / Operations / Credit / Split when terminal ≥ spec | `VisibleWorkspace`, `WorkspaceSpec` (`workspace.rs`) |
| **Tab pane** | Single-tab body: dashboard, positions, settings, … | `Tab`, `PaneSpec`, `PaneHintMode` (`pane.rs`) |
| **Secondary focus** | Sub-region within a tab (e.g. Settings section / Health strip) | `SecondaryFocus`, `SettingsSection`, `SettingsHealthFocus` |
| **Overlays** | Help, logs, tree, detail popup, command palette, toasts | `InputMode` variants, `layered_chrome_active` |

Input is resolved **top-down**: the highest active overlay’s `InputMode` wins; otherwise tab + secondary focus; workspace only affects *visibility* and *workspace tab cycling*, not raw keymaps by itself.

---

## 3. Focus & modes (today, consolidated)

- **`InputMode`** (`app.rs`): authoritative *which keymap / swallow rules* apply. Precedence in `App::input_mode()` is strict (help → detail → palette → credential → … → normal).
- **`AppMode`** (`mode.rs`): coarse **NAV / EDIT / VIEW** for status bar and discoverability; derived from `InputMode` via `app_mode_for_input_mode` — do not duplicate parallel truth.
- **`ModeContext::determine_mode`**: useful for tests and future auto-mode; production path should stay aligned with `input_mode` mapping.
- **`SecondaryFocus`**: only non-`None` on **Settings** today; extend when another tab gains a two-level focus (e.g. split lists).

---

## 4. Workspaces vs split pane

- **`visible_workspace()`** uses `last_main_area_size` + `active_tab` + `split_pane` to decide `VisibleWorkspace`.
- **Invariant:** workspace **must not** switch while an **Edit** `InputMode` holds text field focus without an explicit UX decision (today: overlays generally block workspace switches).

---

## 5. Scroll & selection ownership

- **`ScrollableTableState`**: own `selected` + `scroll` per logical table; call `clamp_to_len` when data length changes; use `adjust_scroll` after layout when viewport height known.
- **Rule:** panels **do not** store ad hoc `usize` scroll fields for table-like UIs; migrate stragglers under **T-1774817606420931000**.

---

## 6. Target “central router” (migration direction)

**Today:** dispatch is split across `input.rs`, `input_settings.rs`, `input_tabs.rs`, `input_views.rs`, `mouse.rs` — workable but spreads precedence knowledge.

**Target (incremental):**

1. **`FocusContext`** (`focus_context.rs`, `App::focus_context()`): today carries `(input_mode, active_tab, secondary_focus)`; extend with `visible_workspace` / overlay depth when the central router lands.
2. **Per-mode handlers** return `Consumed | Bubble` so shared keys (e.g. `Esc`, `?`) behave consistently.
3. **Pane hints** (`PaneHintMode`) generated from `PaneSpec` + `SecondaryFocus` + `InputMode` in one helper to feed `render_hint_bar`.

**Today:** `discoverability::context_hints_for(&FocusContext)` drives compact status-bar hint strips and includes Settings → Health nested focus (`↑/↓` transport vs services).

No big-bang rewrite: introduce helpers and shrink `match` duplication tab-by-tab.

---

## 7. Verification (manual regression checklist)

Run after focus/router/workspace/layout changes. Short-terminal Settings: **`docs/SETTINGS_SHORT_TERMINAL_QA.md`**.

- [ ] **Layer stack:** Help, detail, palette, credential modal, and other overlays consume keys before the tab body; `App::input_mode()` precedence unchanged.
- [ ] **Tabs:** Tab / Shift-Tab; digit mnemonics; in a composed workspace, Tab cycles **inner** panes when the workspace hint is active.
- [ ] **Workspaces:** `visible_workspace()` matches layout; min-size hints if applicable; no silent workspace switch while edit-style modes hold focus (see §4).
- [ ] **Settings:** Section list vs Health sub-focus (transport vs services); keys match hint strip and help overlay.
- [ ] **Scroll:** Tables use `ScrollableTableState`; `clamp_to_len` after snapshot/resize.
- [ ] **Mouse:** Tab bar click regions still correct; Market workspace pane hit-testing matches render splits (wheel routes to pane under cursor).
- [ ] **Narrow terminal:** Spot-check Settings stacked/wide thresholds and Operations column bias.
- [ ] **Toasts / status:** Mode-transition toasts; transport health does not spam (see `app_tests.rs`).

Related task ids (historical): **T-1774817606353005000**, mouse tab regions **T-1774965243417103000**.

---

## 8. Related docs

- `docs/TUI_ARCHITECTURE.md` — file layout, main loop.
- `docs/TUI_RATATUI_INTERACT.md` — sub-focus for charts / orders / palette.
- `docs/TUI_INPUT_DECISION.md` — input policy.
