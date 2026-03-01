# Scripts directory audit: obsolete and unification candidates

**Date:** 2025-03-01

## Implementation status (2025-03-01)

- **2.1 Python test capture:** Done. Single script `run_tests_capture.py` with `--simple`, `--timeout`, `-o`; removed `run_tests_simple.py` and `run_tests_and_capture.py`.
- **2.2 Stale docs:** Done. Single script `update_stale_docs.py` with `--files`, `--docs-dir`, `--days-threshold`, `--jobs` (default 8); removed `update_stale_docs_direct.py` and `update_stale_docs_parallel.py`.
- **2.3 System info:** Done. `collect_system_info.sh` is a thin wrapper around `collect_system_info_python.py`.
- **Obsolete:** Removed `update_tasks_direct.py`, `update_tasks_to_review.py`. Moved `test_tws_connection.cpp` to `native/tests/test_tws_connection.cpp` (CMake updated). Moved `notebooklm_create_*.txt` to `docs/notebooklm/`.
- **Shell deduplication:** Merged `quick_install_test.sh` into `test_repo_install.sh`: `test_repo_install.sh` now supports `--install` (when run as root it runs `install_deb_repo.sh`). Removed `quick_install_test.sh` (it had a hardcoded path and duplicated status logic).

---

## Summary

- **Obsolete / one-off:** 6 items
- **Unite (merge into one script):** 4 groups
- **Keep as-is:** Coverage trio, Exarp wrappers (by design), service start/stop scripts

---

## 1. Obsolete or one-off (candidates for removal or archiving)

| Script | Reason |
|--------|--------|
| `run_tests_simple.py` | Subset of `run_tests_capture.py` (pytest python/tests only, no integration, no exit code). Superseded by capture variant. |
| `run_tests_and_capture.py` | Overlaps with `run_tests_capture.py` (same purpose; capture has uvx fallback, timeout, same output file). Keep one. |
| `update_tasks_direct.py` | One-off: hardcoded task IDs, direct edit of `.todo2/state.todo2.json`. Same list as `update_tasks_to_review.py`. Safe to remove if migration is done. |
| `update_tasks_to_review.py` | One-off: generates MCP update commands for same hardcoded IDs. Pair with above; remove or archive once no longer needed. |
| `test_tws_connection.cpp` | C++ source in `scripts/`; belongs in `native/tests/` or a small test binary. Move or remove. |
| `notebooklm_create_*.txt` (6 files) | NotebookLM prompt templates. If unused, move to `docs/notebooklm/` or remove. |

---

## 2. Unite (merge into a single script or entrypoint)

### 2.1 Python test capture (high value)

**Current:** Three scripts do “run pytest and write output to a file”:

- `run_tests_and_capture.py` – pytest, `test_results.txt`
- `run_tests_capture.py` – uvx/pytest fallback, timeout, `test_results.txt` (referenced in `docs/SESSION_HANDOFF.md`)
- `run_tests_simple.py` – pytest `python/tests/` only, `test_output.txt`, no exit code

**Recommendation:** Keep **one** script, e.g. `run_tests_capture.py`, and extend it with:

- Optional `--simple` (python/tests only, no integration).
- Single output file (e.g. `test_results.txt`).
- Always propagate exit code.

Then remove `run_tests_and_capture.py` and `run_tests_simple.py`, and point any docs/automation at the single script.

### 2.2 Stale docs updater (high value)

**Current:** Two scripts share almost the same logic:

- `update_stale_docs_direct.py` – direct/sequential
- `update_stale_docs_parallel.py` – parallel (ProcessPoolExecutor)

**Recommendation:** One script with a `--parallel` (or `--jobs N`) flag; default can be parallel with a small job count. Same CLI (e.g. accept list of files or “find stale” behavior).

### 2.3 System info collection (medium value)

**Current:**

- `collect_system_info.sh` – bash, macOS-focused (with Linux), outputs JSON. Referenced in `docs/DEVELOPMENT_ENVIRONMENT.md`.
- `collect_system_info_python.py` – Python, cross-platform, richer.

**Recommendation:** Prefer the Python version as the single implementation. Add a small `collect_system_info.sh` that calls `collect_system_info_python.py` so existing docs and `./scripts/collect_system_info.sh` keep working. Optionally deprecate the bash-only logic.

### 2.4 Task status update (low priority, one-off)

**Current:**

- `update_tasks_direct.py` – edits `state.todo2.json` directly.
- `update_tasks_to_review.py` – emits MCP update commands for same IDs.

**Recommendation:** If you still need this workflow, one script with `--mode direct|mcp` and a single shared task-ID list (or config file). Otherwise remove both after the migration is done.

---

## 3. Do not merge (intentional split)

| Scripts | Reason |
|---------|--------|
| `generate_coverage.sh`, `generate_cpp_coverage.sh`, `generate_python_coverage.sh` | Orchestrator + C++ and Python backends; `generate_coverage.sh` calls the other two. Documented in README and TEST_COVERAGE_SETUP. |
| `automate_docs_health_v2.py`, `automate_todo2_duplicate_detection.py`, `automate_todo2_alignment_v2.py` | Exarp entry points; called by `exarp_daily_automation_wrapper.py`. Same pattern, different tools. |
| `exarp_fix_documentation_links.py` + `automate_documentation_link_fixing.py` | Wrapper vs core; wrapper imports core. Keep both. |
| `test_nats_e2e.sh`, `test_nats_e2e_flow.sh`, `test_nats_integration.sh` | Different scope: full e2e vs message flow vs integration/mock. Keep separate. |
| `start_*_service.sh` / `stop_*_service.sh` | Per-service; unifying would add complexity for little gain. |

---

## 4. Other notes

- **Legacy / unreferenced Python:** `process_tasks_parallel.py` is not referenced anywhere (parallel Todo2 task processor). Candidate for removal or documentation as optional.
- **Duplicated logic:** `exarp_validate_docs_format.py` reimplements the validation logic from `validate_docs_format.py` instead of importing it (same REQUIRED_FIELDS, ValidationError, find_entries). Core script is `validate_docs_format.py` (used by pre-commit and docs); Exarp wrapper could be refactored to call/import it.
- **Config/log files in `scripts/`:** `*.json` configs (`docs_health_config.json`, `todo2_alignment_config.json`, etc.) and `*.log` / `.docs_health_history.json` are fine in `scripts/` if automation expects them there; consider moving logs to `logs/` or `.logs/` if you want a single place for logs.
- **`shortcuts/`:** `run_tests.sh` (ctest) vs top-level `run_tests.sh` (ShellSpec) are different; keep both, ensure names/docs are clear.
- **`run_python_tests.sh`** – separate from the Python capture scripts; keep for “run Python tests” without capture.

---

## 5. Suggested action order

1. **Quick wins:** Unify the three “run pytest and capture” scripts into one; optionally remove or archive the two one-off task-update scripts and move/remove `test_tws_connection.cpp` and `notebooklm_create_*.txt` if unused.
2. **Next:** Merge `update_stale_docs_direct.py` and `update_stale_docs_parallel.py` into one script with `--parallel`.
3. **Optional:** Make `collect_system_info.sh` a thin wrapper around `collect_system_info_python.py` and deprecate duplicate bash logic.

---

## 6. Shell scripts vs CMake (duplication)

Several shell scripts repeat configure/build logic that already exists in `CMakePresets.json` or in `native/CMakeLists.txt`. Prefer presets and `cmake --build` to keep a single source of truth.

### 6.1 Clear duplication (use presets)

| Script | Issue | Recommendation |
|--------|--------|----------------|
| **`release_x86.sh`** | Runs `cmake -S ... -B build/macos-x86_64-release`… This matches preset **`macos-x86_64-release`** exactly. | Use `cmake --preset macos-x86_64-release` and `cmake --build --preset`. **(Done.)** |
| **`build_fast.sh`** | Previously custom `build-fast` and manual Intel/TWS builds. | Refactored to use presets `*-release-sccache` / `*-release-ccache`; CMake builds vendor deps. **(Done.)** |
| **`build_distributed.sh`** | Previously custom `build-distributed`, `make -C`. | Refactored to use presets `*-release-distcc` / `*-release-sccache` / `*-release-ccache` and `cmake --build`. **(Done.)** |

### 6.2 Thin wrappers (no change needed)

These already delegate to CMake presets: `build_universal.sh`, `build_with_logging.sh`, `build_ramdisk.sh`, `shortcuts/run_build.sh`, `shortcuts/run_tests.sh`, `integration_test.sh`.

### 6.3 Special cases

- **`setup_worktree.sh`** – Verify step uses raw `cmake -S . -B build`; could use `cmake --preset <host-preset>`.
- **`create_deb_repo.sh`** – Tries preset then fallback; packaging, keep as-is.
- **`generate_cpp_coverage.sh`** – Coverage flags; no coverage preset; keep.
- **`test_nats_e2e.sh`** – `cmake -B build -DENABLE_NATS=ON`; could be a preset with `ENABLE_NATS=ON`.
