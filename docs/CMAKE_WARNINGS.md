# CMake configure warnings

## Current status

**Project-level (our code):** Addressed in `native/CMakeLists.txt`:

- **CMP0135** set to NEW (FetchContent URL extraction timestamps).
- **CMP0167** set to OLD (keep using FindBoost until migration to BoostConfig).
- **Boost**: `Boost_NO_WARN_NEW_VERSIONS ON` to avoid "New Boost version may have incorrect or missing dependencies".
- **nlohmann_json**: `DOWNLOAD_EXTRACT_TIMESTAMP TRUE` in `fetchcontent_declare`.

So **no CMake warnings** come from our own CMake code when configuring without `-Wno-dev`.

---

## Warnings from FetchContent dependencies

When you run:

```bash
cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Debug -DCMAKE_WARN_DEPRECATED=ON
```

you may still see **developer/deprecation warnings** from third-party code fetched by FetchContent. Those are outside our control unless we patch or upgrade the upstream projects.

| Source | Warning | Cause |
|--------|---------|--------|
| **CLI11** (`_deps/cli11-src/CMakeLists.txt`) | ~~Deprecation: compatibility with CMake &lt; 3.10 will be removed~~ | **Patched** via `PATCH_COMMAND` in `native/CMakeLists.txt` (script: `scripts/patch_cmake_minimum.cmake`) |
| **Eigen3** (`_deps/eigen3-src/CMakeLists.txt`) | ~~Same as above~~ | **Patched** via same script (supports `VERSION x.y` and `x.y.z`) |
| **Eigen3 test** | CMP0167 not set (FindBoost module removed) | Eigen’s `find_package(Boost)` |
| **Eigen3 unsupported/test** | CMP0146 not set (FindCUDA module removed) | Eigen’s `find_package(CUDA)` |

These do **not** affect the build or our code; they are in dependency CMake scripts.

**CLI11 and Eigen3** are patched at fetch time via `scripts/patch_cmake_minimum.cmake`: their `cmake_minimum_required(VERSION ...)` is set to **3.10** (i.e. require CMake ≥ 3.10), so the "Compatibility with CMake < 3.10" deprecation is removed. On a **fresh** configure (or after removing `build/_deps/cli11-*` and `build/_deps/eigen3-*`), the patch runs and the warning no longer appears for them.

---

## Suppressing dependency warnings

To get a quiet configure (no dev/deprecation warnings from subprojects):

```bash
cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Debug -Wno-dev
```

The Ansible role `roles/ib_box_spread` already uses `-Wno-dev` when it runs CMake.

---

## Checking for new warnings

To see all current CMake warnings (including from dependencies):

```bash
cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Debug -DCMAKE_WARN_DEPRECATED=ON 2>&1 | grep -E "Warning|deprecat|CMP0"
```

To focus on our code only, run the same configure **without** `-Wno-dev` and look for messages that reference `native/CMakeLists.txt` (there should be none after the above fixes).
