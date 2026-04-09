# TUI shortcuts on macOS terminals

**Audience:** Operators running `tui_service` in **Terminal.app**, **iTerm2**, or similar macOS terminal emulators.

The TUI reads keys through **crossterm**. On macOS, **⌘ (Command)** is exposed as the **Super** modifier, so the same chords documented in the in-app help overlay and in `discoverability.rs` apply when the terminal forwards them to the application (see `input/router.rs`).

---

## Terminal.app vs iTerm2

| Topic | Terminal.app | iTerm2 |
|--------|----------------|--------|
| **⌘ chords** | Usually delivered to the shell/TUI unless a profile captures them for the emulator (rare for letter/number chords). If a chord does nothing, check **Terminal → Settings → Profiles → Keyboard** for custom key bindings that steal the sequence. | **Profiles → Keys** controls how modifiers are reported. Ensure ⌘ combinations are sent to the session, not remapped to menu actions, if you rely on TUI ⌘ shortcuts. |
| **Option (⌥) as Meta** | For bindings that need **Meta/Alt** (not used for the ⌘ table below), enable **Use Option as Meta key** in the profile’s **Keyboard** pane so ⌥ sends an escape prefix instead of inserting special characters. | Under **Profiles → Keys**, set **Left/Right Option key** to **Esc+** or **Meta** when you need Alt/Meta semantics for future bindings or copied Linux muscle memory. |
| **Quirks** | Some keys produce different bytes depending on keyboard layout and “Unicode input” settings. | **Natural Text Editing** preset vs custom key maps can change how ⌥←/⌥→ behave; that mostly affects line editing, not the global ⌘ chords listed here. |

If **⌘⇧P** or **⌘⇧T** conflict with the terminal’s own shortcuts, remap the terminal’s shortcut or use the non-⌘ alternatives (`:` for the command palette, **Ctrl+T** for theme cycle).

---

## Common ⌘ chords (TUI)

These match the command palette labels in `discoverability.rs` and macOS routing in `input/router.rs` (`handle_macos_cmd_key` plus the ⌘⇧ branch). **Shift** is only required where shown (⌘⇧P, ⌘⇧T).

| Chord | Action |
|--------|--------|
| **⌘Q** | Quit |
| **⌘W** | Close detail / dismiss overlays (same as `CloseDetail` — clears detail popup, help, log panel, tree panel) |
| **⌘,** | Jump to **Settings** tab |
| **⌘/** | Open **help** overlay (on macOS this is routed to help; plain **`/`** elsewhere is often chart/order filter) |
| **⌘⇧P** | Toggle **command palette** |
| **⌘⇧T** | **Cycle UI theme** (session override: `default` ↔ `high_contrast`; same as **Ctrl+T**) |
| **⌘R** | Refresh / force snapshot (`ForceSnapshot`) |
| **⌘P** | Toggle **split pane** |
| **⌘0** … **⌘9** | Jump to tab by index (same mapping as digit keys `0`–`9`: **0** = Settings, **1** = Dashboard, **2** Positions, **3** Charts, **4** Orders, **5** Alerts, **6** Yield, **7** Loans, **8** Discount Bank, **9** Ledger) |

**Non-⌘ equivalents** (all platforms): **`?`** help, **`:`** palette, **`q`** quit, **`p`** split, **`f`** refresh (or FMP on Dashboard/Positions per `input_shell.rs`), digit keys for tabs, **`m`** mode cycle toast.

---

## Paper vs live (status bar only)

The TUI status bar shows **read-only** Alpaca connectivity hints (**A** = paper, **P** = live) using snapshot-backed status colors — not a keyboard shortcut. Credential routing and environment setup are described in workspace/settings docs (for example `workspace.rs` / Settings **Data sources**); do not put API secrets in shell history or config snippets you paste into chat.

---

## See also

- **[`TUI_ARCHITECTURE.md`](./TUI_ARCHITECTURE.md)** — Help overlay behavior and maintainer notes.
- **`agents/backend/services/tui_service/src/discoverability.rs`** — Command palette registry and key labels.
- **`agents/backend/services/tui_service/src/input/router.rs`** — macOS ⌘ dispatch order (⌘⇧ before plain ⌘).
