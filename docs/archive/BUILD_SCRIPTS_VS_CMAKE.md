# Build Scripts vs CMake/Ninja

What belongs in shell scripts vs CMake/Ninja, and what could be moved to reduce duplication.

## Scripts vs CMake: No Logic Moved Into CMake

**Audit result:** No script functionality has been moved *into* CMake. Root `CMakeLists.txt` only adds convenience targets that **invoke** scripts:

| CMake target       | Invokes                               |
|--------------------|----------------------------------------|
| `fetch_third_party`| `scripts/fetch_third_party.sh`         |
| `lint`             | `scripts/run_linters.sh`               |
| `exarp-lint`       | `scripts/run_exarp_go_tool.sh` lint     |
| `exarp-list`       | `scripts/run_exarp_go_tool.sh` --list  |

Lint/build/test logic stays in scripts or in native CMake (C++ targets, CTest). Scripts that do build/clean/test **delegate** to `cmake --build` / `ctest`; they do not duplicate CMake logic.

**Clean:** `build_portable.sh` and `build_universal.sh` already use `cmake --build --preset X --target clean`. **Exception:** `build_ramdisk.sh` `clean_build()` uses `rm -rf "${BUILD_DIR}"/*`; it could use `cmake --build "${BUILD_DIR}" --target clean` for consistency (keeps CMakeCache, so no reconfigure needed). **Justfile** `clean` does `rm -rf build/*` (nuclear: all presets); use when you want to wipe every build dir.

**Test:** `scripts/run_tests.sh` runs **ShellSpec** (shell script tests). CTest (C++ tests) is run by Justfile `test`, `build_portable.sh test`, `build_universal.sh test`, `scripts/shortcuts/run_tests.sh`. No overlap.

---

## Already in CMake / Presets

- **Preset inventory and script mapping** – See `docs/planning/CMAKE_PRESETS_VS_SCRIPTS.md` for which presets exist and which scripts use them; testPresets and binaryDir conventions are documented there.
- **Preset selection** – `CMakePresets.json` defines all presets (debug/release, sccache/ccache/distcc, ramdisk, -ai). Scripts only choose which preset to use.
- **Compiler launcher** – `ENABLE_SCCACHE` / `ENABLE_CCACHE` in presets; `native/CMakeLists.txt` sets `CMAKE_CXX_COMPILER_LAUNCHER`. No need for scripts to set it.
- **Build directory** – `binaryDir` in presets (including `build-ramdisk`). Scripts that used to override `-B` can rely on presets; `build_ai_friendly.sh` already prefers `-ramdisk` preset when `build-ramdisk` exists.
- **Clean** – `cmake --build --preset X --target clean`. Scripts can thin out to just document or call that instead of `rm -rf build/*`.
- **Test** – `ctest --preset X`. Scripts wrap it; no logic to move.
- **TWS API / Intel decimal** – CMake uses `ExternalProject` or vendor paths; scripts don’t need to run separate `cmake -S third_party/...` unless we want a dedicated “build deps only” flow.

## Good Candidates to Move or Centralize

### 1. Parallel jobs (`-j`)

**Current:** Scripts do `-j$(nproc)` or `-j$(sysctl -n hw.ncpu)`.

**Move:** Set `CMAKE_BUILD_PARALLEL_LEVEL` in the environment or in a single wrapper (e.g. `scripts/shortcuts/run_build.sh`). Alternatively use CMake 3.28+ build preset `jobs` if we bump `cmakeMinimumRequired`. Then scripts can drop `-j` and just run `cmake --build --preset X`.

**Action:** Document in README/AGENTS: “For parallel build, set `export CMAKE_BUILD_PARALLEL_LEVEL=$(nproc)` (or `sysctl -n hw.ncpu` on macOS) or pass `-j N` to the script.” Optionally add to `run_build.sh` / `build_fast.sh` so one place sets it.

### 2. Third-party fetch hint when configure fails

**Current:** Scripts source `ensure_third_party.sh` before configure; it runs `fetch_third_party.sh` or exits with a message.

**Move (partial):** In `native/CMakeLists.txt`, when TWS API (or Intel decimal) is required and not found, `message(FATAL_ERROR "... Run: ./scripts/fetch_third_party.sh")` so the first configure failure is self-explanatory. Scripts can then make `ensure_third_party` optional for advanced users who run `cmake --preset` directly.

**Action:** Add a clear `FATAL_ERROR` (or early `return()`) with the fetch command when vendor dirs are missing. Optionally add a CMake target that runs the fetch script so `cmake --build <dir> --target fetch_third_party` is possible (convenience only).

### 3. “Fetch then reconfigure” retry (e.g. build_ramdisk.sh)

**Current:** build_ramdisk.sh detects “missing libtwsapi” in build output and runs `fetch_third_party.sh` then reconfigure and retry.

**Move:** Rely on (2): configure fails with a clear “run fetch_third_party” message. User runs `./scripts/fetch_third_party.sh` then re-runs configure/build. We can add a `fetch_third_party` custom target that runs the script, and document “if configure fails, run: cmake --build <dir> --target fetch_third_party (or ./scripts/fetch_third_party.sh) then reconfigure.” Then remove the retry logic from build_ramdisk.sh to avoid duplication.

## Should Stay in Scripts

- **Ramdisk create/unmount** – `setup_ramdisk.sh`, `workspace_ram_disk_manager.sh`: need `hdiutil`, `diskutil`, mount points. Not CMake.
- **Preset selection by platform** – Detecting arch/OS and choosing a preset name is caller/orchestration; keep in one or two scripts (e.g. `build_ai_friendly.sh`, `run_build.sh`) and have others call them or accept a preset name.
- **sccache/ccache cache config** – `SCCACHE_DIR`, `ccache --max-size`, etc., are environment/setup. Document in README; optional one-time setup script is fine. CMake only needs the launcher.
- **AI-friendly JSON wrapper** – `build_ai_friendly.sh` runs build, parses log, emits JSON. Orchestration and parsing belong in a script; CMake already provides `BUILD_AI_FRIENDLY` for compiler flags.
- **System package install** – `ensure_third_party.sh` running `brew install boost` / `apt-get install` is environment setup; CMake can only fail with a message and optionally a target that runs a script.
- **Logging / tee / timestamps** – build_with_logging.sh, etc.: convenience; keep in scripts.

## Summary Table

| Concern              | Currently in      | Move to CMake/Ninja? | Note                                      |
|----------------------|------------------|----------------------|-------------------------------------------|
| Presets / binaryDir  | CMakePresets.json| Done                 | Ramdisk presets added.                    |
| Compiler launcher    | CMakeLists + preset | Done              | ENABLE_SCCACHE/CCACHE.                    |
| Parallel jobs       | Scripts (-j)     | Optional             | Env or one script; or preset `jobs` later. |
| Third-party fetch    | ensure_third_party.sh | Partial          | Add FATAL_ERROR + optional fetch target.  |
| Retry after fetch   | build_ramdisk.sh | Remove               | Rely on clear configure error + doc.      |
| Clean / test        | Scripts + ctest  | Already CMake        | Scripts can just invoke preset.           |
| Ramdisk create      | setup_ramdisk.sh | No                   | OS-specific.                             |
| AI JSON wrapper     | build_ai_friendly.sh | No               | Orchestration + parsing.                  |

## Recommended Next Steps

1. **Document** – In README or AGENTS: “To build: `cmake --preset <name>` then `cmake --build --preset <name>`. Set `CMAKE_BUILD_PARALLEL_LEVEL` for parallelism. If configure fails for missing TWS/Intel deps, run `./scripts/fetch_third_party.sh` then reconfigure.”
2. **CMake** – Add a clear `message(FATAL_ERROR "TWS API not found. Run: ./scripts/fetch_third_party.sh")` (or equivalent) when vendor dirs are missing and required. Optionally add `add_custom_target(fetch_third_party COMMAND ${CMAKE_SOURCE_DIR}/scripts/fetch_third_party.sh)` in the top-level or native CMakeLists so `cmake --build <dir> --target fetch_third_party` is available.
3. **Scripts** – Simplify build_ramdisk.sh: remove the “if build fails with missing libtwsapi then fetch and retry” block; rely on configure failing with the message above. Optionally set `CMAKE_BUILD_PARALLEL_LEVEL` in build_fast.sh / run_build.sh so `-j` is not required in every script.
