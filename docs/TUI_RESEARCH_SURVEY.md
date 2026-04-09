# TUI stack — dependency survey

**Last updated:** 2026-04-09  
**Scope:** Versions and upstream documentation for the `tui_service` terminal stack. Source manifest: [`agents/backend/services/tui_service/Cargo.toml`](../agents/backend/services/tui_service/Cargo.toml). **Resolved** patch versions below match `cargo tree -p tui_service --depth 1` in this repo (refresh after `Cargo.lock` changes).

---

## 1. Version index

| Crate | Manifest (`Cargo.toml`) | Resolved (lock / `cargo tree`) | Notes |
|--------|-------------------------|----------------------------------|--------|
| **ratatui** | `0.30` | **0.30.0** | Default backend feature pulls in `ratatui-crossterm`; service also depends on **crossterm** directly for `event-stream`. |
| **crossterm** | `0.28` + `features = ["event-stream"]` | **0.28.1** | Pin aligns with Ratatui’s crossterm integration line; **0.28.1** is not yanked (some earlier `0.28.0` publishes were yanked on crates.io — use the resolved patch). |
| **tokio** | Workspace: `tokio = { version = "1", … }` in [`agents/backend/Cargo.toml`](../agents/backend/Cargo.toml); `tui_service` adds `signal` | **1.50.0** | Features used by `tui_service`: `rt-multi-thread`, `macros`, `sync`, `time`, `signal`. |

**Workspace Rust edition:** `2021` in `[workspace.package]` — there is **no** workspace-level `rust-version` / MSRV field in the backend `Cargo.toml`.

---

## 2. Upstream documentation (pinned versions)

### ratatui 0.30.x

| Resource | URL |
|----------|-----|
| Project site (concepts, tutorials, highlights) | [https://ratatui.rs/](https://ratatui.rs/) |
| API docs (0.30.0) | [https://docs.rs/ratatui/0.30.0/ratatui/](https://docs.rs/ratatui/0.30.0/ratatui/) |
| Crate page | [https://crates.io/crates/ratatui/0.30.0](https://crates.io/crates/ratatui/0.30.0) |
| Repository | [https://github.com/ratatui/ratatui](https://github.com/ratatui/ratatui) |
| Release highlights (0.30) | [https://ratatui.rs/highlights/v030/](https://ratatui.rs/highlights/v030/) |

### crossterm 0.28.x

| Resource | URL |
|----------|-----|
| API docs (0.28.1) | [https://docs.rs/crossterm/0.28.1/crossterm/](https://docs.rs/crossterm/0.28.1/crossterm/) |
| Crate page | [https://crates.io/crates/crossterm/0.28.1](https://crates.io/crates/crossterm/0.28.1) |
| Repository | [https://github.com/crossterm-rs/crossterm](https://github.com/crossterm-rs/crossterm) |

### tokio 1.x (resolved 1.50.0)

| Resource | URL |
|----------|-----|
| Guide & overview | [https://tokio.rs/](https://tokio.rs/) |
| Tutorial | [https://tokio.rs/tokio/tutorial](https://tokio.rs/tokio/tutorial) |
| API docs (1.50.0) | [https://docs.rs/tokio/1.50.0/tokio/](https://docs.rs/tokio/1.50.0/tokio/) |
| Repository | [https://github.com/tokio-rs/tokio](https://github.com/tokio-rs/tokio) |

---

## 3. MSRV / Rust version notes

**Repo:** The backend workspace does **not** declare `package.rust-version`. Toolchains are otherwise governed by developer/CI choice.

**Per-crate minimum Rust** (from [crates.io](https://crates.io/) API `rust_version` for the resolved releases — useful as a **lower bound the dependency authors support**, not necessarily what CI uses):

| Crate @ resolved | crates.io `rust_version` |
|------------------|---------------------------|
| ratatui 0.30.0 | **1.86.0** |
| tokio 1.50.0 | **1.71** |
| crossterm 0.28.1 | **1.63.0** |

**Practical implication:** Building `tui_service` with **ratatui 0.30** requires a **recent stable** toolchain (at least the maximum of the above, currently **1.86** per Ratatui’s published metadata). If CI or contributors use an older `rustc`, resolution or compile errors will surface here first.

---

## 4. Maintenance signal (lightweight)

| Crate | Signal |
|-------|--------|
| **ratatui** | Very high download volume; active release train (0.30 late 2025); docs site + book-style material; GitHub org maintenance. Trust/audit metadata present on crates.io for 0.30.0. |
| **crossterm** | Mature, widely depended-on terminal layer; **0.29+** exists upstream — this service stays on **0.28.x** for compatibility with the Ratatui/crossterm stack until a deliberate upgrade. |
| **tokio** | De-facto standard async runtime for Rust; frequent patch releases under the **1.x** API stability promise; maintained under **tokio-rs**. |

---

## 5. Related internal docs

- [`TUI_IMPLEMENTATION_BACKLOG.md`](./TUI_IMPLEMENTATION_BACKLOG.md) — planning index (§5 links here).
- [`TUI_ARCHITECTURE.md`](./TUI_ARCHITECTURE.md), [`TUI_RATATUI_INTERACT.md`](./TUI_RATATUI_INTERACT.md) — architecture and optional `ratatui-interact` pilots (separate pinned versions in the same `Cargo.toml`).

---

## 6. Refresh checklist

After changing `tui_service` or workspace dependencies:

1. `cd agents/backend && cargo tree -p tui_service --depth 1` — confirm **ratatui** / **crossterm** / **tokio** lines.
2. Update the **Resolved** column and doc links if major/minor bumps occur.
3. Re-check crates.io `rust_version` for new releases if MSRV regressions matter for CI.
