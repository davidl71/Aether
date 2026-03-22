# Development Tools

## Platform and crate audits

- **[RUST_CRATE_OPPORTUNITIES_AUDIT.md](platform/RUST_CRATE_OPPORTUNITIES_AUDIT.md)** — Places where hand-rolled or duplicated logic could use existing crates (JSONC, backoff, config, mocks). Implemented items: `jsonc-parser` (api, tui_service), `backoff` (tui_service circuit breaker, nats_adapter DLQ).
- **[STUB_CODE_PLANNING.md](platform/STUB_CODE_PLANNING.md)** — Stub/placeholder audit: what can be implemented with no questions (e.g. FMP fundamentals routes) vs human review (ib_adapter scope, runtime_state placeholders, TUI event routing).

## Code Navigation

- **ctags** - Generate tags for code navigation

  ```bash
  ctags -R .
  ```

- **tagref** - Check cross-references in code comments

  ```bash
  brew install tagref
  # Run check
  tagref --upstream-refs .
  ```

  Format: `# [tag:tagname]` and `# [ref:tagname]`

- **xrefcheck** - Check cross-references in documentation (markdown)

  ```bash
  cargo install xrefcheck
  # Run check
  xrefcheck -i docs/
  ```

## Rust build size and disk usage

- **[RUST_BUILD_SIZE.md](platform/RUST_BUILD_SIZE.md)** — What we did to shrink the Rust `target` (quant without Polars, nats_adapter features), what drives the remaining ~2–2.5 GiB, and why further feature-flag trimming saves little.

**sccache for Rust:** `just build-rust`, `just test`, and `./scripts/build_rust_ai_friendly.sh` use sccache when it is on `PATH` (same cache as C++; see [RUST_BUILD_SIZE.md § Many builds](platform/RUST_BUILD_SIZE.md#many-builds-separate-target-dirs-and-caching)).

**cargo-sweep** prunes old build artifacts while keeping the latest.

- **Install:** `cargo install cargo-sweep`
- **Automated stamp:** Running `just test`, `just build-rust`, or `just sanity` updates a timestamp after success (if cargo-sweep is installed), so `just sweep` only removes artifacts older than your last good build.
- **Prune manually:** `just sweep` (or `just sweep-dry` to preview). To prune by age: `just sweep-auto` (default: 14 days) or `just sweep-auto 7`.
- **Scheduled prune:** e.g. cron daily: `0 2 * * * cd /path/to/repo && just sweep-auto 14`

See `just sweep-stamp`, `just sweep`, `just sweep-dry`, `just sweep-auto` in the root Justfile.

**Unused dependencies (cargo-udeps):** For a compile-time accurate report, install [cargo-udeps](https://crates.io/crates/cargo-udeps) and use nightly: `rustup install nightly` then `just udeps` (or `cd agents/backend && rustup run nightly cargo udeps`). The workspace must build successfully. To ignore false positives, add `[package.metadata.cargo-udeps.ignore]` in the crate's Cargo.toml.

**Global Cargo cache (cargo-cache):** Manage `~/.cargo` (registry, git checkouts). `just cache` shows size; `just cache-trim` trims to 500M; `just cache-autoclean` removes source checkouts to free space. Requires: `cargo install cargo-cache`.

## Installation

```bash
# macOS
brew install universal-ctags tagref

# Rust tools
cargo install xrefcheck cargo-sweep cargo-cache
```
