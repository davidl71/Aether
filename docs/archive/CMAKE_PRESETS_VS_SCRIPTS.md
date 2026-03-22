# CMake Presets vs Build Scripts

Audit of `CMakePresets.json` vs scripts that select or use presets. Kept in sync when adding presets or scripts.

## Preset inventory (CMakePresets.json)

### Configure presets (by category)

| Preset name | binaryDir | Condition / notes |
|-------------|-----------|-------------------|
| **Base (debug/release)** | | |
| macos-arm64-debug | build/macos-arm64-debug | |
| macos-arm64-release | build/macos-arm64-release | |
| macos-x86_64-debug | build/macos-x86_64-debug | |
| macos-x86_64-release | build/macos-x86_64-release | |
| macos-universal-debug | build/macos-universal-debug | DEPRECATED |
| macos-universal-release | build/macos-universal-release | DEPRECATED |
| linux-x64-debug | build/linux-x64-debug | Linux |
| linux-x64-release | build/linux-x64-release | Linux |
| linux-aarch64-debug | build/linux-aarch64-debug | Linux |
| linux-aarch64-release | build/linux-aarch64-release | Linux |
| ubuntu-x64-debug | build/ubuntu-x64-debug | Linux |
| ubuntu-x64-release | build/ubuntu-x64-release | Linux |
| windows-x64-debug | build/windows-x64-debug | Windows |
| windows-x64-release | build/windows-x64-release | Windows |
| **Cache / distributed** | | |
| macos-arm64-release-sccache | build/macos-arm64-release-sccache | |
| macos-x86_64-release-sccache | build/macos-x86_64-release-sccache | |
| linux-x64-release-sccache | build/linux-x64-release-sccache | Linux |
| macos-arm64-release-ccache | build/macos-arm64-release-ccache | |
| macos-x86_64-release-ccache | build/macos-x86_64-release-ccache | |
| linux-x64-release-ccache | build/linux-x64-release-ccache | Linux |
| macos-arm64-release-distcc | build/macos-arm64-release-distcc | |
| macos-x86_64-release-distcc | build/macos-x86_64-release-distcc | |
| linux-x64-release-distcc | build/linux-x64-release-distcc | Linux |
| **AI-friendly (JSON diagnostics)** | | |
| macos-arm64-debug-ai | build/macos-arm64-debug-ai | inherits macos-arm64-debug |
| macos-x86_64-debug-ai | build/macos-x86_64-debug-ai | inherits macos-x86_64-debug |
| linux-x64-debug-ai | build/linux-x64-debug-ai | Linux, inherits linux-x64-debug |
| **Ramdisk (binaryDir = build-ramdisk)** | | |
| macos-arm64-debug-ramdisk | **build-ramdisk** | macOS only |
| macos-x86_64-debug-ramdisk | **build-ramdisk** | macOS only |
| macos-arm64-debug-ai-ramdisk | **build-ramdisk** | macOS only |
| macos-x86_64-debug-ai-ramdisk | **build-ramdisk** | macOS only |

### Build presets

All configure presets above have a matching build preset (same name, `configurePreset` = self).

### Test presets

**Before fix:** testPresets did **not** include `-ai` or `-ramdisk` presets, so `ctest --preset macos-arm64-debug-ramdisk` (e.g. from `build_ramdisk.sh test`) would fail with “preset not found” in the test presets list.

**After fix:** testPresets include macos-arm64-debug-ai, macos-x86_64-debug-ai, linux-x64-debug-ai, macos-arm64-debug-ramdisk, macos-x86_64-debug-ramdisk, macos-arm64-debug-ai-ramdisk, macos-x86_64-debug-ai-ramdisk so that `build_ramdisk.sh test` and AI-friendly/ramdisk flows can run ctest.

---

## Script → preset mapping

| Script | Preset selection | Notes |
|--------|------------------|--------|
| **build_portable.sh** | `choose_preset()`: Darwin arm64→macos-arm64-{debug\|release}, x86_64→macos-x86_64-*, Linux x64→linux-x64-*, aarch64→linux-aarch64-*. Default **release**. Override: `CMAKE_PRESET`, `--debug`/`--release`. | Does not support Windows; uses only base presets. |
| **build_ramdisk.sh** | When `build-ramdisk` exists: `ramdisk_preset()` → macos-arm64-debug-ramdisk or macos-x86_64-debug-ramdisk. Else no override (build_portable default). | Thin wrapper; delegates configure/build/test/clean to build_portable.sh. |
| **build_ai_friendly.sh** | `detect_default_preset()` then `resolve_preset()` (prefer -ai), then `prefer_ramdisk_if_setup()` (prefer -ramdisk if build-ramdisk exists). Override: `CMAKE_PRESET`. | Uses -ai and -ai-ramdisk presets when available. |
| **build_fast.sh** | `detect_preset_suffix()` → macos-arm64, macos-x86_64, or linux-x64. Then `PRESET=${SUFFIX}-release-sccache` or `-release-ccache`. | Uses sccache/ccache presets only. |
| **build_distributed.sh** | Same suffix; `PRESET=${SUFFIX}-release-distcc` (or -sccache / -ccache). | Uses distcc/sccache/ccache presets. |
| **build_universal.sh** | Default macos-arm64-release or macos-x86_64-release. Override: `CMAKE_PRESET`. | Single-arch release; not universal binary. |
| **build_with_logging.sh** | First arg or default `macos-arm64-debug`. | Any preset by name. |
| **build_variant.sh** | First arg or default `macos-arm64-debug`. | Any preset by name. |
| **shortcuts/run_build.sh** | `detect_default_preset()` (macos-arm64-debug, macos-x86_64-debug, linux-x64-debug). Override: second arg or `CMAKE_PRESET`. | Debug-oriented. |
| **shortcuts/run_tests.sh** | Same default as run_build. Override: first arg or `CTEST_PRESET`. | Requires matching testPreset. |
| **setup_platform_settings.sh** | macos-arm64-debug, macos-x86_64-debug, linux-x64-debug, windows-x64-debug by platform. | One preset per platform. |
| **create_deb_repo.sh** | ubuntu-x64-release else linux-x64-release. | Linux only. |
| **release_x86.sh** | Hardcoded macos-x86_64-release. | |
| **integration_test.sh** | Default macos-x86_64-release; override `CMAKE_PRESET`. **BUILD_DIR** = `build/${PRESET}` except for -ramdisk (see below). | Uses ctest --preset; needs correct BUILD_DIR for CLI path. |
| **run_integration_tests.sh** | Validates macos-arm64-debug / linux-x64-debug (configure only). | |

---

## binaryDir convention

- **Standard:** `build/<preset-name>` (e.g. `build/macos-arm64-debug`).
- **Ramdisk:** All `-ramdisk` presets use `binaryDir: "build-ramdisk"` (symlink or dir created by setup_ramdisk.sh).

Scripts that derive the build directory from the preset name must treat `-ramdisk` specially: use `build-ramdisk` instead of `build/${PRESET}`. **integration_test.sh** does this so `CLI_BIN` points at the correct binary when `CMAKE_PRESET=macos-arm64-debug-ramdisk`.

---

## Alignment checklist

- [x] **testPresets** – All configure presets used for `ctest --preset` in scripts have a matching testPreset (including -ai and -ramdisk).
- [x] **BUILD_DIR** – integration_test.sh uses `build-ramdisk` when preset name ends with `-ramdisk`, else `build/${PRESET}`.
- [x] **build_portable** – choose_preset() only returns base presets; -ramdisk is injected by build_ramdisk.sh via CMAKE_PRESET.
- [x] **build_fast / build_distributed** – Preset names match CMakePresets.json (`<suffix>-release-sccache`, etc.).

---

## Adding a new preset

1. Add **configurePreset** (and optionally **buildPreset**) in `CMakePresets.json`.
2. If any script will run `ctest --preset <name>`, add a **testPreset** with the same name and `configurePreset` set.
3. If the preset uses a non-standard **binaryDir** (e.g. not `build/<name>`), document it here and update any script that computes the build dir from the preset name (e.g. integration_test.sh).

---

## Related

- `docs/planning/BUILD_SCRIPTS_VS_CMAKE.md` – What lives in scripts vs CMake; clean/test/ramdisk.
- `CMakePresets.json` – Single source of truth for preset names and binaryDir.
