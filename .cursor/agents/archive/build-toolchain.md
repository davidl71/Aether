# Agent A: Build & Toolchain

## Role

Fix C++ toolchain issues and build vendored third-party libraries so the main project can compile.

## Tasks

1. **Fix C++ toolchain - Xcode Command Line Tools headers** (`T-1772135684202624000`)
   - The C++ standard library headers (e.g. `<string>`) are not found by clang++
   - Headers exist at `/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/include/c++/v1/` but the compiler searches `/Library/Developer/CommandLineTools/usr/include/c++/v1/` which has partial headers
   - Fix: run `xcode-select --install` or `sudo rm -rf /Library/Developer/CommandLineTools && xcode-select --install`
   - Verify: `echo '#include <string>' | clang++ -x c++ -std=c++20 -fsyntax-only -`

2. **Build Intel Decimal libbid.a from vendored source** (`T-1772135684174386000`)
   - Source at `native/third_party/IntelRDFPMathLib20U2/LIBRARY/`
   - Has `RUNOSX` and `RUNOSXINTEL64` build scripts
   - Build for the current architecture (arm64)
   - Verify: `file native/third_party/IntelRDFPMathLib20U2/LIBRARY/libbid.a` shows the archive
   - Add a Justfile recipe `build-intel-decimal` under the `# --- Build ---` section

3. **Mark TWS API update as done** (`T-1772135684080505000`)
   - Already cloned to `native/third_party/tws-api/` (version 10.44.01)
   - Run: `exarp-go-task_workflow(action="update", task_id="T-1772135684080505000", status="Done")`

## Files You Own (exclusive)

- `native/third_party/IntelRDFPMathLib20U2/` (build artifacts)
- System toolchain (`xcode-select`)
- `Justfile` -- ONLY the `build-intel-decimal` recipe (add below `build-deps`)

## Files You Must NOT Touch

- `native/src/` or `native/include/` (owned by Agent E)
- `scripts/` (owned by Agent B)
- `ansible/` (owned by Agent C)
- `proto/` (owned by Agent D)

## Completion Criteria

- [ ] `clang++ -std=c++20` can find `<string>`, `<vector>`, `<optional>`
- [ ] `libbid.a` exists and is a valid archive for the current architecture
- [ ] All three exarp tasks marked Done
