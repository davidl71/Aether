# Wishlist / Deferred Decisions

## IDE Extensions – C++ Language Server Choice

- Decision: Choose one C/C++ extension to standardize on (Microsoft “C/C++” vs Anysphere “C/C++”).
- Rationale: Running both causes duplicate diagnostics and slower IntelliSense. Microsoft’s cpptools integrates tightly with CMake Tools; Anysphere may be sufficient for some workflows.
- Status: Deferred
- Trigger to decide:
  - After cross‑platform verification (macOS ARM64/x86_64, Windows, Linux)
  - Post initial debugging sessions on each platform
  - Once team preference on IntelliSense behavior and performance is clear

- Temporary guidance: Do not enable both simultaneously in the same workspace.

## Debugger Choice on Windows

- Decision: Standardize on MSVC debugger (Visual Studio/VSCode) vs LLDB/CodeLLDB for MinGW/Clang.
- Status: Deferred
- Trigger to decide: After first successful end‑to‑end debug session on Windows with the chosen toolchain.

# Wishlist

## High Priority

- [ ] Evaluate Lean CLI integration for optional QuantConnect/LEAN workflows
  - Track lessons learned in `docs/LEAN_LEARNINGS.md`
  - Reference Lean CLI documentation: <https://www.lean.io/docs/v2/lean-cli>
  - Gate any implementation on clear benefit for our IBKR box-spread flow

## Low Priority

- [ ] Universal binary support (x86_64 + arm64)
  - Currently builds x86_64 only due to TWS API library architecture limitations
  - Would require rebuilding TWS API library and all dependencies as universal binaries
  - Priority: Low - x86_64 works fine on Intel Macs and via Rosetta on Apple Silicon
  - Note: Deprecated universal presets in `CMakePresets.json` still exist but build x86_64 only
