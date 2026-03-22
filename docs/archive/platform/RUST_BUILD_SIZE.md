# Rust backend: build size and shrink options

How we reduced the Rust workspace build size and what drives the remaining ~2–2.5 GiB in `agents/backend/target`. Reference for future dependency or feature changes.

---

## Current state

- **Workspace:** `agents/backend/` (Cargo workspace).
- **Typical `target` size** after a full debug build: **~2–2.5 GiB** (down from ~24 GiB before the changes below).
- **Largest artifacts:** `libibapi`, `libasync_nats`, `libapi`, `librustls`, `libtokio`, `libsqlx_sqlite`, `libreqwest`, plus support crates.

---

## Changes made to shrink the build

### 1. Quant crate: remove RustQuant and Polars

- **Before:** `quant` depended on **RustQuant** (and transitively **Polars**), which pulled in a large dataframe/ML stack and blew `target` up to ~24 GiB.
- **After:** Replaced with minimal in-crate implementations:
  - **BSM** (`crates/quant/src/bsm.rs`): Black–Scholes–Merton price, delta, gamma, theta, vega, rho, implied volatility (bisection); inline normal CDF, no extra deps.
  - **GBM** (`crates/quant/src/gbm.rs`): GBM Euler–Maruyama paths for Monte Carlo; only `rand` (and dev `rand_chacha`).
- **Result:** Polars and RustQuant removed from the dependency tree; `target` dropped to ~2 GiB for a full build.

### 2. NATS adapter: trim async-nats features and TLS backend

- **`crates/nats_adapter/Cargo.toml`** — `async-nats`:
  - **Disabled `object-store`** — we only use NATS KV and JetStream in `collection_aggregation.rs`; object-store was unused.
  - **Switched TLS to `ring`** — use `ring` for TLS/crypto instead of `aws-lc-rs` to avoid the heavy `aws_lc_sys` dependency (native build and large rlib).
- **Result:** Smaller dependency tree and faster builds for `nats_adapter`; no behavioral change for current NATS usage.

### 3. CI: cargo-sweep to trim cached artifacts

- **`.github/workflows/agents-backend-rust.yml`:** After “Run cargo audit”, we install **cargo-sweep** and run `cargo sweep sweep . --time 0` so the cached `target` is smaller.
- **Local:** Optional `just udeps` (or `cargo udeps` with nightly) to check unused deps; `cargo sweep` to remove old artifacts.

---

## What drives the remaining size

Approximate largest rlibs (debug build):

| Crate / area        | Rough size  | Notes |
|---------------------|------------|-------|
| **ibapi**           | ~36 MB     | IBKR API binding; no feature flags to trim. Would require a different adapter or fork to shrink. |
| **async_nats**      | ~30+ MB    | Already trimmed (no object-store, ring for crypto). |
| **rustls**          | ~20+ MB    | Used by **reqwest** (HTTP) and **async-nats** (TLS). Cannot remove without dropping NATS TLS or changing the NATS TLS stack. |
| **tokio**           | ~19+ MB    | Workspace uses explicit features: `rt-multi-thread`, `macros`, `sync`, `time`; some crates add `signal` or `fs`. Not using `full`. |
| **sqlx** (sqlite)   | ~19+ MB    | Already minimal: `runtime-tokio-native-tls`, `sqlite` only (plus chrono/uuid in ledger). |
| **reqwest**         | ~17 MB     | Already `default-features = false` with `cookies`, `json`, `rustls`, `query`. |

---

## Feature-flag options and expected savings

We evaluated further trimming via Cargo features:

| Area      | Current state | Possible change | Expected savings |
|-----------|----------------|------------------|-------------------|
| **Tokio** | Explicit features (no `full`); some crates add `signal` or `fs`. | Drop `signal` where not needed (e.g. tui_service, backend_service). | **Very small** (single-digit MB). |
| **Reqwest** | `default-features = false`, `rustls` for TLS. | Switch to `native-tls` instead of `rustls`. | **Little or none.** rustls is still required by async-nats (tokio-rustls), so rustls stays in the graph; you add native-tls and a C TLS dependency. |
| **SQLx**  | Already minimal: `runtime-tokio-native-tls`, `sqlite` (and chrono/uuid in ledger). | — | **None.** |

**Conclusion:** Further feature-flag tweaks yield at most a few MB. Meaningful savings would require replacing or slimming **ibapi** or changing the NATS/HTTP TLS story (e.g. a different NATS client or TLS backend).

---

## Many builds: separate target dirs and caching

If you run many builds (e.g. frequent `cargo build` / `cargo test` plus occasional release or CI-style builds), the following can help.

### What Cargo already does

- **One `target/` per workspace** — `agents/backend/target/` holds both `debug/` and `release/`; they don’t overwrite each other. Incremental artifacts are per profile, so switching between `cargo build` and `cargo build --release` does not wipe the other.
- **Incremental compilation** — Within the same profile, only changed crates are recompiled.

### When separating build targets helps

Using a **different `CARGO_TARGET_DIR`** is useful when:

| Scenario | Approach | Benefit |
|----------|----------|--------|
| **CI vs local** | `CARGO_TARGET_DIR=target/ci cargo build --release` (e.g. in CI) vs default `target/` locally | CI doesn’t contend with or wipe your local incremental state; local stays warm for dev. |
| **Parallel workflows** | Two terminals: one with `CARGO_TARGET_DIR=target/a`, the other `target/b` | Both can build at once without file locking on the same dir. Downside: two full target trees (more disk), no shared incremental cache. |
| **Ramdisk** | `CARGO_TARGET_DIR=/Volumes/IBBoxSpreadDev/caches/cargo-target` (see `scripts/setup_ram_optimization.sh`, `docs/RAM_OPTIMIZATION_GUIDE.md`) | Puts Rust artifacts on a RAM disk for faster I/O; same single dir, just different location. |

Separating by profile (e.g. one dir for debug, one for release) is **not** recommended: Cargo already keeps them separate inside `target/debug` and `target/release`, and splitting into two top-level dirs would duplicate dependency builds and use more disk.

### Caching: sccache for Rust

The project uses **sccache** for C++ (e.g. `build_fast.sh`). **Rust** builds use sccache when it is on `PATH`:

| Entry point | Behavior |
|-------------|----------|
| **`./scripts/build_rust_ai_friendly.sh`** | Sources `workspace_paths.sh` (sets `SCCACHE_DIR` to `.cache/sccache`), then sets `RUSTC_WRAPPER=sccache` if `sccache` is found; same cache as C++. |
| **`just build-rust`**, **`just test`**, **`just sanity`** | Run `cargo` with `RUSTC_WRAPPER=sccache` and `SCCACHE_DIR=../.cache/sccache` when `sccache` is found. |
| **`./scripts/run_linters.sh`** | Rust step runs `cargo fmt --check` and `cargo clippy`; uses sccache when on `PATH` (same `SCCACHE_DIR` as workspace_paths). |
| **`agents/backend/scripts/run-tests.sh`** | Sets `RUSTC_WRAPPER=sccache` and `SCCACHE_DIR` when sccache is found before fmt/clippy/test. |
| **CI (agents-backend-rust.yml)** | Installs sccache and sets `RUSTC_WRAPPER`/`SCCACHE_DIR` when install succeeds (optional; speeds up clippy/test). |
| **Ad-hoc `cargo build`** | Set once: `export RUSTC_WRAPPER=sccache` (and optionally `SCCACHE_DIR` to share with C++). |

- **First build:** Same as without sccache; results are stored in `SCCACHE_DIR` (or `~/.sccache` if unset).
- **Later builds:** Unchanged crates can be served from cache (replay stored compiler output), so rebuilds are faster.
- **CI:** Set `RUSTC_WRAPPER=sccache` and use a shared sccache backend (e.g. Redis, S3) so CI and local can share cache.

Rust support in sccache is built-in; no extra config beyond `RUSTC_WRAPPER`. The same `SCCACHE_DIR` used for C++ can be used for Rust.

### Trimming old artifacts

- **cargo-sweep** — Already used in CI and available locally (`just sweep`, `docs/DEVELOPMENT_TOOLS.md`). Removes old artifacts from `target/` while keeping the latest, reducing disk use when you perform many builds over time.

---

## References

- **Workspace and parallelism:** [BUILD_PARALLELIZATION_AND_MODULARITY.md](../BUILD_PARALLELIZATION_AND_MODULARITY.md)
- **Rust crate opportunities:** [RUST_CRATE_OPPORTUNITIES_AUDIT.md](RUST_CRATE_OPPORTUNITIES_AUDIT.md)
- **NATS usage:** [NATS_API.md](NATS_API.md), [NATS_KV_USAGE_AND_RECOMMENDATIONS.md](NATS_KV_USAGE_AND_RECOMMENDATIONS.md)
