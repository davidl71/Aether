# TUI Text Input Decision (`tui-input`)

**Date**: 2026-03-31  
**Decision**: **Hybrid** — adopt `tui-input` for *text-entry pockets*, keep the current action router + simple buffers elsewhere  
**Status**: Active guidance

## Context

Aether’s TUI is an operator console with a keyboard-first interaction model and
an explicit `Action` router (`services/tui_service/src/input.rs`). Today, most
“text input” surfaces are implemented as ad-hoc `String` buffers plus a small
set of keybindings (e.g. backspace, enter, esc, up/down navigation).

This decision is about whether to **adopt** the `tui-input` crate, use a **hybrid**
approach, or **pass** and keep custom buffers.

## Pilot outcome (what we learned)

- The current approach is *sufficient* for short, constrained buffers (paths,
  small filters, single-field edits).
- Where we want “editor-ish” behavior (cursor movement, insert at cursor,
  delete word, home/end, etc.), custom `String` buffers become noisy and
  inconsistent across tabs.
- The existing input architecture already cleanly separates:
  - key parsing → `Action`
  - `Action` → state mutation in focused modules (`input_settings.rs`,
    `input_loans.rs`, …)
  So we can adopt `tui-input` without rewriting global navigation.

## Decision

### Hybrid (selected)

Use `tui-input` **only** for UI elements that benefit from cursor-aware editing
and standardized editing requests. Keep the current explicit action router as
the top-level input system.

**What this means in practice**:

- Keep `Action` as the command vocabulary and preserve the existing
  mode-specific routing (`InputMode`, tab handlers).
- For a given “text entry” surface, replace the raw `String` buffer with
  `tui_input::Input` and translate keystrokes into `InputRequest`s.

## Trade-offs

### Pros

- **Consistency**: shared editing semantics across tabs (cursor, insert, delete).
- **Lower per-feature cost**: new text fields don’t require re-implementing
  editing behavior.
- **Scoped adoption**: avoids a disruptive, cross-module refactor (no “big bang”
  input rewrite).

### Cons / risks

- **Another dependency** in `tui_service` (versioning, upgrades, API drift).
- **Two styles of input** (simple buffers vs `tui-input`) until rollout completes.
- **Keybinding policy choices**: need to decide which editing keys are supported
  consistently (minimum viable set vs full readline-like).

## Rollout order (if/when adopting)

Start where users feel editing pain and where behavior is isolated, then expand:

1. **Settings credential entry** (`settings_credential_buffer`): needs cursor
   movement + safe editing; easy to scope.
2. **Loans bulk import path** (`loan_import_path`): pragmatic test surface for
   single-line input, max-length, validation toasts.
3. **Chart search / command palette text entry**: improves navigation UX without
   touching data-path logic.
4. **Settings: add-symbol + config edit**: unify remaining small text-entry flows.

## Alternatives considered

- **Pass (custom `String` buffers everywhere)**:
  - Works for minimal needs, but repeated editing logic will keep accumulating.
- **Adopt ratatui-interact’s `Input`/`TextArea` broadly**:
  - More disruptive; better revisited as part of a larger focus/component
    consolidation, not as a text-entry-only change.
- **Use prompt/dialog crates (`tui-prompts`)**:
  - Better for modal workflows; less suitable for inline, always-visible inputs.

## References

- `tui-input` docs: `https://docs.rs/tui-input/`
- `tui-input` repo: `https://github.com/sayanarijit/tui-input`
- Aether research: `docs/research/ratatui-interact-evaluation.md`
- **Field vs list sub-focus** (optional `tui-interact` feature, `FocusManager`): `docs/TUI_RATATUI_INTERACT.md` — orthogonal to `tui-input`; covers Tab order between a buffer and an adjacent list in overlays.

