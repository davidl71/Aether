# Build Parallelization and Modularity

How the project parallelizes builds and how its components are split into modules for incremental and parallel work.

---

## Build Parallelization

### C++ / CMake (Ninja)

- **Generator:** Presets use **Ninja** (`CMakePresets.json`). Ninja runs compile and link jobs in parallel by default.
- **Job count:** Controlled by **`CMAKE_BUILD_PARALLEL_LEVEL`** (or Ninja’s `-j` when using `cmake --build` with `-j N`).
- **Default in scripts:** `scripts/build_fast.sh`, `scripts/shortcuts/run_build.sh`, `scripts/build_ai_friendly.sh`, `scripts/build_portable.sh`, and `scripts/build_universal.sh` source `scripts/include/set_parallel_level.sh`, which sets `CMAKE_BUILD_PARALLEL_LEVEL` to the number of CPUs when unset (macOS: `sysctl -n hw.ncpu`, Linux: `nproc`, fallback: `4`).
- **Override:** `export CMAKE_BUILD_PARALLEL_LEVEL=8` (or any number) before running the build.
- **Docs:** AGENTS.md and [BUILD_SCRIPTS_VS_CMAKE.md](planning/BUILD_SCRIPTS_VS_CMAKE.md) describe this; no need to pass `-j` when using the wrapper scripts.

### Rust (Cargo)

- **Parallelism:** Cargo compiles **workspace crates in parallel** and, within a crate, compiles units in parallel up to the number of CPUs.
- **Job control:** `CARGO_BUILD_JOBS` (default: number of CPUs). Example: `CARGO_BUILD_JOBS=4 cargo build`.
- **No script override:** The project does not set `CARGO_BUILD_JOBS`; use the env var when you need to limit or increase parallelism.

### Lint (run_linters.sh)

- **Parallel mode:** `./scripts/run_linters.sh --parallel` or `LINT_PARALLEL=1 ./scripts/run_linters.sh` run independent linters (cppcheck, clang analyze, Infer, SwiftLint, Bandit, Ruff, ESLint, stylelint, typecheck, etc.) **in parallel**. Exarp-go and shellcheck run afterward in sequence.
- **Default:** Sequential. Use `--parallel` for faster full lint runs when you have enough cores.

### Tests

- **C++ (CTest):** CTest can run test executables in parallel; see CTest documentation and `ctest -j N`.
- **Rust:** `cargo test` runs tests in parallel by default; control with `cargo test -- --test-threads=N`.
- **Python:** `pytest -n auto` (with pytest-xdist) for parallel test runs if the project adds it.

---

## Modularity

### C++ (removed)

- **Status:** The C++ native build (`native/`) has been removed. Root `CMakeLists.txt` no longer includes it; the repo is Rust-first. See [MULTI_LANGUAGE_CODEBASE.md](MULTI_LANGUAGE_CODEBASE.md).
- **Linters:** `run_linters.sh` still has cppcheck/clang/infer steps that run only when a C++ tree exists (`native/src` or top-level `src`); with none present, they are skipped.

### Rust (agents/backend)

- **Workspace:** Single Cargo workspace in `agents/backend/Cargo.toml` with **member crates**:
  - `crates/common`, `crates/market_data`, `crates/strategy`, `crates/risk`, `crates/api`, `crates/ledger`, `crates/nats_adapter`, `crates/quant`, `crates/ib_adapter`, `crates/discount_bank_parser`
  - `services/backend_service`, `services/tui_service` (binaries)
- **Modularity:** Each crate is a separate compilation unit. Cargo builds the dependency graph and compiles crates in parallel where possible. Changing one crate only rebuilds that crate and its dependents.
- **Build one binary:** `cargo build -p backend_service` (or `cargo run -p backend_service`). Build all: `cargo build`.
- **Build size:** For how we reduced `target` size and what drives the remaining ~2–2.5 GiB, see [RUST_BUILD_SIZE.md](platform/RUST_BUILD_SIZE.md).
- **Many builds:** For when to use a separate `CARGO_TARGET_DIR`, sccache for Rust (`RUSTC_WRAPPER=sccache`), and cargo-sweep, see [RUST_BUILD_SIZE.md § Many builds](platform/RUST_BUILD_SIZE.md#many-builds-separate-target-dirs-and-caching).

### Python

- **Layout:** `python/` is a flat tree of modules and apps (TUI, integration, services, etc.). No formal multi-package workspace; dependencies are in `requirements.txt` / `pyproject.toml` at repo or `python/` level.
- **Incremental:** No native “build” step; running tests or the app uses the current source. Cython-built extensions (if enabled) are produced by the C++/CMake build.

### Scripts and orchestration

- **Build entry points:** Rust: `just build-rust`, `./scripts/build_rust_ai_friendly.sh`. Legacy C++ scripts (e.g. `build_fast.sh`) and CMake presets remain for reference but target a removed native tree. See [BUILD_SCRIPTS_VS_CMAKE.md](planning/BUILD_SCRIPTS_VS_CMAKE.md).
- **Lint:** Single script `run_linters.sh` (Rust fmt+clippy, Python, shell, Ansible, etc.) with optional `--parallel`.
- **Clean:** Rust `just clean-rust` or `cargo clean` in `agents/backend`.

---

## Quick reference

| Layer    | Parallelization control              | Modular unit / incremental build        |
|----------|--------------------------------------|----------------------------------------|
| Rust     | Cargo (default) or `CARGO_BUILD_JOBS`| Workspace crates; Cargo DAG             |
| Lint     | `--parallel` / `LINT_PARALLEL=1`     | Per-linter; script runs many in parallel|
| Tests    | `cargo test`, pytest                 | Per test binary / test run              |

For preset naming, cache (sccache/ccache), and third-party fetch, see [BUILD_SCRIPTS_VS_CMAKE.md](planning/BUILD_SCRIPTS_VS_CMAKE.md), [CMAKE_PRESETS_VS_SCRIPTS.md](planning/CMAKE_PRESETS_VS_SCRIPTS.md), and AGENTS.md.
