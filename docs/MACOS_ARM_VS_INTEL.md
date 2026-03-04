# macOS: ARM (Apple Silicon) vs Intel

How the project distinguishes and supports **Apple Silicon (arm64)** and **Intel (x86_64)** on macOS.

---

## Detect architecture

```bash
uname -m
# arm64   → Apple Silicon (M1/M2/M3/M4, etc.)
# x86_64  → Intel Mac
```

Scripts use this to choose the right CMake preset (e.g. `build_portable.sh`, `build_ai_friendly.sh`, `build_ramdisk.sh`).

---

## Presets and build dirs

| Architecture | Debug preset            | Release preset             | Build dir                    |
|-------------|--------------------------|----------------------------|------------------------------|
| **ARM64** (Apple Silicon) | `macos-arm64-debug`       | `macos-arm64-release`       | `build/macos-arm64-debug` etc. |
| **x86_64** (Intel)        | `macos-x86_64-debug`      | `macos-x86_64-release`      | `build/macos-x86_64-debug` etc. |

CMake sets `CMAKE_OSX_ARCHITECTURES` to `arm64` or `x86_64` per preset. Each architecture has its own build directory so you can keep arm64 and x86_64 builds side by side.

---

## When to use which

- **Same-arch build (normal):** On an M-series Mac use `macos-arm64-*`; on an Intel Mac use `macos-x86_64-*`. Scripts pick this automatically from `uname -m`.
- **Cross-arch (e.g. Intel binary on Apple Silicon):** Run configure and build with the Intel preset:
  ```bash
  cmake --preset macos-x86_64-debug
  cmake --build --preset macos-x86_64-debug
  ```
  The binary will be in `build/macos-x86_64-debug/bin/` (runs under Rosetta 2 on Apple Silicon).
- **Universal binary (arm64 + x86_64 in one binary):** Use `scripts/build_universal.sh` (or the build target that builds both and lipo-combines). Not the default; use when you need a single fat binary.

---

## Script defaults

| Script / behavior | ARM64 default | Intel default |
|-------------------|---------------|---------------|
| `build_portable.sh` | `macos-arm64-release` (or debug with `--debug`) | `macos-x86_64-release` |
| `build_ai_friendly.sh` | `macos-arm64-debug` (or `-ai`, `-ramdisk` if set up) | `macos-x86_64-debug` |
| `build_ramdisk.sh` | `macos-arm64-debug-ramdisk` | `macos-x86_64-debug-ramdisk` |
| Cursor commands (e.g. build:debug) | Often `macos-arm64-debug` | Use `macos-x86_64-debug` if on Intel |

Override any time with `CMAKE_PRESET=macos-x86_64-debug` (or the preset you want).

---

## Path differences between architectures

Paths that **differ by architecture** vs **shared**:

### Build and binary paths (differ by arch)

| What | ARM64 | Intel (x86_64) |
|------|--------|-----------------|
| Debug build dir | `build/macos-arm64-debug` | `build/macos-x86_64-debug` |
| Release build dir | `build/macos-arm64-release` | `build/macos-x86_64-release` |
| CLI binary (debug) | `build/macos-arm64-debug/bin/ib_box_spread` | `build/macos-x86_64-debug/bin/ib_box_spread` |
| TUI binary (debug) | `build/macos-arm64-debug/bin/ib_box_spread_tui` | `build/macos-x86_64-debug/bin/ib_box_spread_tui` |
| Tests | `build/macos-arm64-debug/` (ctest) | `build/macos-x86_64-debug/` (ctest) |

Same pattern for **-ai**, **-sccache**, **-ccache**, **-distcc** presets: path always includes the preset name, e.g. `build/macos-arm64-release-sccache`, `build/macos-x86_64-release-sccache`.

### Ramdisk build (shared path, one arch at a time)

When using a **-ramdisk** preset, both arm64 and x86_64 use the **same** build directory:

| What | Path (both arches) |
|------|---------------------|
| Build dir | `build-ramdisk` (symlink to `/Volumes/IBBoxSpreadBuild/build`) |
| CLI binary | `build-ramdisk/bin/ib_box_spread` |

So `build-ramdisk` holds only one architecture at a time (whichever preset you last configured: `macos-arm64-debug-ramdisk` or `macos-x86_64-debug-ramdisk`). Switch arch by reconfiguring with the other preset.

### Paths that do not differ by arch

- **Cache RAM disk** (`setup_ram_optimization.sh`): `/Volumes/IBBoxSpreadDev/caches/` — same for both; no arch in path.
- **Rust `CARGO_TARGET_DIR`** (when using ramdisk env): e.g. `/Volumes/IBBoxSpreadDev/caches/cargo-target` — same for both.
- **Project dirs** (config, scripts, proto, native source): no arch-specific paths.
- **Cursor/Just commands**: Some commands hardcode `build/macos-arm64-debug`; on Intel use `CMAKE_PRESET=macos-x86_64-debug` and run the binary from `build/macos-x86_64-debug/bin/...`, or change the command to your preset’s path.

### Scripts that derive build dir from preset

- **integration_test.sh**: `BUILD_DIR=build/${PRESET}` except for `-ramdisk` presets, where `BUILD_DIR=build-ramdisk`. So `CLI_BIN=${BUILD_DIR}/bin/ib_box_spread` is correct for any preset.
- **build_fast.sh**, **build_distributed.sh**: Set `BUILD_DIR` from the chosen preset (e.g. `build/macos-arm64-release-sccache`), so binary path follows arch.

### Quick reference: binary path by preset

```text
build/<preset-name>/bin/ib_box_spread
build/<preset-name>/bin/ib_box_spread_tui
```

Examples: `build/macos-arm64-debug/bin/ib_box_spread`, `build/macos-x86_64-release/bin/ib_box_spread`, `build-ramdisk/bin/ib_box_spread` (when using a -ramdisk preset).

### OS and homedir

**OS (same machine)**  
On a single Mac, the OS is the same whether you build or run arm64 or x86_64: **macOS (Darwin)**. `uname -s` is `Darwin` for both. The only runtime difference is **Rosetta 2** when you run an x86_64 binary on Apple Silicon. There are no separate “ARM OS” vs “Intel OS” paths; the split is **build/output** paths (e.g. `build/macos-arm64-*` vs `build/macos-x86_64-*`).

**HOME / homedir (same machine)**  
On that same Mac, **`$HOME` is the same** for both architectures (e.g. `/Users/yourname`). So these do **not** differ by arch:

- `~/.cargo` (registry, git, config)
- `~/.cache` (pip, etc.)
- `~/.ccache`, `~/.sccache`
- Project path if under `$HOME` (e.g. `~/Projects/.../ib_box_spread_full_universal`)

Homedir can differ **by OS** (e.g. macOS `/Users/...` vs Linux `/home/...`) or **by machine/user**; it does not change when you switch between building for arm64 vs x86_64 on the same login. Scripts that use `$HOME` or `~` (including `setup_ram_optimization.sh` when it links `~/.cargo`, `~/.ccache`, etc.) see the same homedir for both arches.

---

## Integration tests and CI

- **integration_test.sh** defaults to `macos-x86_64-release`; override with `CMAKE_PRESET`.
- CI / release: use the preset that matches the runner (e.g. macOS Intel runner → `macos-x86_64-*`). Release scripts like `release_x86.sh` target Intel explicitly.

---

## Ramdisk and RAM optimization

RAM disk and cache scripts are architecture-agnostic: the same cache RAM disk and `setup_ram_optimization.sh` apply to both arm64 and x86_64. Build presets that use the ramdisk (`macos-*-debug-ramdisk`) still choose arm64 vs x86_64 via the base preset (e.g. `macos-arm64-debug-ramdisk` vs `macos-x86_64-debug-ramdisk`).

---

## References

- [PORTABLE_BUILD_AND_RUNNER.md](PORTABLE_BUILD_AND_RUNNER.md) — Preset selection and portable build
- [CMakePresets.json](../CMakePresets.json) — All `macos-arm64-*` and `macos-x86_64-*` presets
- [docs/planning/CMAKE_PRESETS_VS_SCRIPTS.md](planning/CMAKE_PRESETS_VS_SCRIPTS.md) — Script vs preset mapping
