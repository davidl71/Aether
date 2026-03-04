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
