# Build failures and dependencies

## Current failure: C++ standard library headers not found

**Symptom:** Build fails with:

- `fatal error: 'mutex' file not found`
- `fatal error: 'string' file not found`
- `fatal error: 'memory' file not found`

These occur when compiling the **TWS API** (and can affect other targets). The compiler is not finding the C++ standard library headers (`<string>`, `<mutex>`, `<memory>`, etc.).

### Cause

On macOS, this usually means **Xcode Command Line Tools (CLT)** are missing, incomplete, or the active developer directory is wrong. The project’s main CMake injects the SDK C++ path when CLT headers are missing; the TWS API sub-build (`native/ibapi_cmake/CMakeLists.txt`) does the same injection so it works when built via ExternalProject or standalone.

### Fix: Install or repair Command Line Tools

1. **Install / repair CLT (recommended first step):**

   ```bash
   xcode-select --install
   ```

   Follow the GUI prompt to install or update Command Line Tools.

2. **If that doesn’t fix it**, reset and reinstall:

   ```bash
   sudo rm -rf /Library/Developer/CommandLineTools
   xcode-select --install
   ```

3. **If you use full Xcode** (not just CLT), ensure it’s selected:

   ```bash
   sudo xcode-select -s /Applications/Xcode.app/Contents/Developer
   ```

4. **Verify the toolchain:**

   ```bash
   ./scripts/verify_toolchain.sh
   ```

   Or manually:

   ```bash
   echo '#include <string>' | clang++ -x c++ -std=c++20 -fsyntax-only -
   ```

   If this fails with `'string' file not found`, CLT/SDK are still not set up correctly.

### After fixing the toolchain

- Reconfigure and build from a clean or existing build dir:

  ```bash
  cmake --preset macos-x86_64-debug   # or your preset
  cmake --build --preset macos-x86_64-debug
  ```

  Or use `just build-portable build` / `just build-keep-going-json`.

---

## Other dependencies (already satisfied if main build progresses)

If the build gets past the TWS API and fails elsewhere, check:

| Dependency    | macOS (Homebrew)        | Linux (apt)           | Notes                    |
|---------------|-------------------------|------------------------|--------------------------|
| CMake (≥3.23) | `brew install cmake`    | `apt install cmake`    | Required                 |
| Ninja         | `brew install ninja`    | `apt install ninja-build` | Required              |
| C++20 compiler| Xcode CLT / Xcode       | `g++-12` or `clang-14` | CLT must have C++ stdlib |
| Protobuf      | `brew install protobuf` | `apt install libprotobuf-dev` | For TWS API / protos |
| Boost         | `brew install boost`    | `apt install libboost-all-dev` | Optional, some features |
| Abseil        | `brew install abseil`   | Optional               | For some builds          |

To fetch vendored third-party (TWS API, Intel decimal) without building:

```bash
./scripts/fetch_third_party.sh
```

---

## Build with “keep going” to see more errors

To have the build continue after the first failure and surface more issues:

```bash
BUILD_KEEP_GOING=1 ./scripts/build_ai_friendly.sh --json-only
# or
just build-keep-going-json
```

Log path: `logs/build_ai_friendly.log`.

---

## Protobuf / TWS API version alignment

**Symptom:** Build or lint fails with:

- `#error "Protobuf C++ gencode is built with an incompatible version of"`
- `error: unknown type name 'PROTOBUF_CONSTEXPR'`

The TWS API ships pre-generated `.pb.h`/`.pb.cc` built with **Protobuf C++ 6.33.4**. If your system uses a different protobuf version (e.g. Homebrew’s `protobuf`), the headers and runtime no longer match.

### What we do

- When using the **vendored GitHub layout** (`source/cppclient/client`), the project builds the TWS API via `ibapi_cmake`, which:
  1. Regenerates `.pb.h`/`.pb.cc` from `source/proto/*.proto` using your system `protoc` (so generated code matches your installed protobuf).
  2. Puts the **generated** include directory **first** so the compiler uses those headers instead of the vendored 6.33.4 ones.
- So you can use **any compatible system protobuf** (e.g. `brew install protobuf`); no need to install Protobuf 6.33.4.

### If you still see version errors

- Ensure `TWS_API_CLIENT_DIR` is passed to the TWS API build (it is when using the vendored GitHub layout).
- Ensure `source/proto` exists under the TWS API vendor root so regeneration can run.
- See `native/ibapi_cmake/CMakeLists.txt`: when `IBAPI_PROTO_GENERATED_DIR` is set, it is listed first in `target_include_directories`.

---

## Summary

1. **First step:** Fix C++ headers with `xcode-select --install` (and, if needed, the reinstall or `xcode-select -s` steps above), then run `./scripts/verify_toolchain.sh`.
2. **Then:** Re-run the build; the TWS API and rest of the tree should get the same C++ header path.
3. **Optional:** Use `BUILD_KEEP_GOING=1` to collect multiple errors in one run.

---

## Market data cache

The legacy memcached backend has been removed from the active native build. The remaining cache support is the optional injected cache interface used by market-data handlers.
