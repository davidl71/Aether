# TUI Pantry workflow (widget development)

This repo’s active TUI lives in `agents/backend/services/tui_service/`. When iterating on a widget, it’s often faster to develop it in isolation (Storybook-style) rather than running the full app.

`tui-pantry` provides a component development loop for **ratatui** widgets.

## Install

Use the upstream docs for the tool itself:

- `https://docs.taho.is/tui-pantry`

Typical install:

```bash
cargo install tui-pantry
```

## Recommended layout for Aether

Keep pantry scaffolding **out of the repo root** to avoid polluting git status.

From the repo root:

```bash
mkdir -p tmp/tui-pantry
cd tmp/tui-pantry
cargo pantry init
```

This creates a `pantry.toml` and starter “ingredients”.

## Wiring Aether widgets into the pantry

1. Add Aether as a path dependency in the pantry’s `Cargo.toml` (or create a small crate that depends on the TUI widget module you want to preview).
2. Expose a widget render function you can call from the pantry “ingredient” entrypoint.

Practical rule of thumb:

- If you’re iterating on a **pure render function** (takes a `Rect` + data and returns a `Widget`/`Text`), prefer exposing it from the module and previewing it directly.
- If it depends on **app state**, create a small, deterministic sample state inside the pantry ingredient.

## Run

```bash
cargo pantry
```

## Notes

- Keep pantry examples **deterministic** (no network calls) so screenshots and regressions are stable.
- Treat pantry as a **dev tool**, not a production dependency: avoid coupling app runtime code to pantry-only types.
