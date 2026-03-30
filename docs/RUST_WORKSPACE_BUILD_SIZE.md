# Rust workspace build-size hot spots (T-1774382071632753000)

**Recorded:** 2026-03-30  
**Workspace:** `agents/backend/` (see `Cargo.toml` `default-members` vs full `members`)  
**Profile measured:** `target/debug` after `cargo build` (default members). Disk figures are **on-disk** totals for an incremental tree that may include historical artifacts.

## Verification commands run

| Step | Command | Result |
|------|---------|--------|
| Build (default members) | `cd agents/backend && cargo build` | OK |
| Tests | `cd agents/backend && cargo test --workspace` | OK |
| Lint (repo gate) | `cargo clippy --workspace --all-targets --all-features -- -D warnings` (same as `scripts/run_linters.sh`) | **Fails** — `clippy::manual_range_contains` in `crates/market_data` (`polygon.rs`, `yield_curve.rs` lib + tests) |

## Disk footprint (summary)

| Path | Size (approx.) |
|------|----------------|
| `agents/backend/target` | **12 GB** |
| `agents/backend/target/debug` | **9.2 GB** |
| `agents/backend/target/debug/deps` | **8.4 GB** |
| `agents/backend/target/release` | **2.8 GB** (present; not rebuilt for this note) |
| `target/debug/incremental` | 0 B (inactive / unused in this tree) |

Large totals include **many duplicate `.rlib` hashes** (different crates, features, and test/bench splits), not a single clean build.

## Primary binaries (debug, `target/debug/`)

| Artifact | Size (KB, `du`) | Notes |
|----------|-----------------|--------|
| `backend_service` | ~62,984 | Axum + `api` + NATS + adapters |
| `cli` | ~47,228 | Pulls broad stack including market/yield tooling |
| `tui_service` | ~34,228 | Ratatui + logger + API client paths |
| `tws_yield_curve_daemon` | ~23,764 | Smaller service binary |
| `libquant.a` | ~12,520 | `quant` publishes `staticlib` / `cdylib` |
| `ib_probe` | ~9,424 | Auxiliary binary if present |
| `libapi.rlib` | ~7,524 | Largest in-tree **workspace** rlib at repo root |

## Third-party crates — largest **single** `.rlib` per crate name (`deps/`)

Method: max `du -k` per `lib<crate>-<hash>.rlib` basename prefix.

| Crate (lock name) | ~Max rlib (KB) | Notes |
|-------------------|----------------|--------|
| `optionstratlib` | 35,277 | **Dev-only** in `quant` (`crates/quant/Cargo.toml` `[dev-dependencies]`). Still compiled for **tests/benches**; dominant “fat” dev dep. |
| `chrono_tz` | 18,977 | Time-zone tables; duplicated across dependents |
| `time_tz` | 17,057 | Same class as above |
| `ibapi` | 17,063 | IBKR client; core to `ib_adapter` |
| `async_nats` | 15,454 | NATS client stack |
| `zerocopy` | 14,381 | Low-level / transitive |
| `simba` | 11,990 | Numeric (nalgebra ecosystem) |
| `tokio` | 11,856 | Async runtime |
| `rustls` | 10,639 | TLS |
| `syn` | 9,782 | Proc-macro build artifact churn in `deps/` |
| `nalgebra` | 9,767 | Linear algebra |
| `yfinance_rs` | 9,178 | Market data provider |
| `regex_automata` | 8,653 | `regex` stack |
| `lopdf` | 8,254 | PDF (`discount_bank_parser` / extraction) |
| `criterion` | 7,962 | **Benches** |
| `api` (workspace) | 7,321 | Snapshot / HTTP read models |

Many rows are **duplicated many times** in `deps/` with different hashes (normal for a long-lived `target/`).

## Default-members boundary vs full workspace

Full `[workspace].members` includes `broker_execution_legacy` and `ib_execution_legacy`. **`default-members` omits both** (see `agents/backend/Cargo.toml`).

- **Effect:** `cargo build` from `agents/backend` **does not** compile those legacy crates unless you pass `-p …` or build from a manifest that selects them.
- **Build-size impact:** Avoids their object code and dependency closure in the default operator/CI path; absolute savings depend on how often those packages were rebuilt before the split.

Other documented boundaries (R sidecar, no `market_data` → `api` dependency cycle) reduce **design risk** and unnecessary coupling; they are not directly reflected as a single directory in `target/` without a before/after clean build comparison.

## Hot-spot themes (actionable)

1. **`target/` hygiene:** Multi-GB trees are often duplicate `deps` artifacts + old release/debug mixes. Periodic `cargo clean` (or separate `CARGO_TARGET_DIR` per CI job) gives predictable measurements.
2. **Dev dependencies:** `optionstratlib` is the largest single optional weight for `quant` test builds — gate behind a feature or move cross-check tests behind `--features` if compile time becomes painful (product decision).
3. **TLS / HTTP / PDF:** `rustls`, `aws_lc_sys`, `hyper`, `reqwest`, `lopdf` — expected for connectivity and PDF parsing; savings require architectural cuts (fewer features, smaller HTTP stack), not micro-opts.
4. **Lint debt:** Repo clippy gate currently blocks `just lint` until `market_data` range checks are fixed.

## Related docs

- `docs/MULTI_LANGUAGE_CODEBASE.md` — Rust workspace location and commands  
- `docs/R_RUST_BOUNDARY.md` — analytics / Rust boundary (orthogonal to `target/` size, complements dependency discipline)
