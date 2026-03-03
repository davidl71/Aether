# Troubleshooting and Fixes (Notes for AI)

This document records fixes and patterns that helped during development, so future AI assistants can avoid the same pitfalls and apply the same solutions.

**Reference:** When debugging TWS/native build or runtime issues, check this file and `docs/TWS_API_COMPARISON_AND_PROTOBUF.md`.

---

## 1. TWS Reconnection Crash ("terminate called without an active exception")

### Symptom

After "Connection closed by TWS", the process crashes with `terminate called without an active exception` during the first reconnection attempt.

### Cause

On reconnect, `start_reader_thread()` was called again and did `reader_thread_ = std::make_unique<std::thread>(...)`, which **replaced** the `unique_ptr` that still held the **previous** reader thread. That previous thread was still running (it had invoked `connectionClosed()` and triggered the reconnect). In C++, destroying a joinable `std::thread` without calling `join()` is undefined behavior and typically calls `std::terminate()`.

### Fix (what helped)

1. **Reader lifecycle mutex**
   Add a `reader_mutex_` and use it whenever creating, joining, or resetting `reader_thread_`.

2. **Signal reader to exit from `connectionClosed()`**
   After setting `connected_ = false`, call `signal_.issueSignal()` so the reader thread wakes from `waitForSignal()`, sees the disconnect, exits its loop, and can be joined safely.

3. **Join before starting a new reader**
   In `start_reader_thread()`: if `reader_thread_` exists and is joinable, signal it to exit (`connected_ = false`, `signal_.issueSignal()`), unlock, `join()` the old thread, lock again, `reader_thread_.reset()`, then create and assign the new thread **under the same mutex**.

4. **Disconnect and timeout paths**
   Use the same join/reset pattern under `reader_mutex_` when stopping the reader on explicit disconnect or connection timeout, so the thread is never destroyed without being joined.

### Code locations

- `native/src/tws_client.cpp`: `reader_mutex_`, `connectionClosed()`, `start_reader_thread()`, `disconnect()`, timeout branch in `run_connection_loop()`.

---

## 2. TWS EWrapper Callback Types (OrderId / TickerId undeclared)

### Symptom

Build fails with errors like:
- `'OrderId' has not been declared`
- `'TickerId' has not been declared`

in `tws_client.cpp` overrides (e.g. `nextValidId`, `tickPrice`, `orderStatus`, `openOrder`).

### Cause

The project’s EWrapper overrides used types `OrderId` and `TickerId`. The **current** TWS API (GitHub `tws-api`, `EWrapper_prototypes.h`) declares these callbacks with **`int`** (e.g. `void nextValidId(int orderId)`, `void tickPrice(int reqId, ...)`). There are no `OrderId`/`TickerId` typedefs in the API we use.

### Fix (what helped)

Replace every callback parameter that used `OrderId` or `TickerId` with **`int`** in `native/src/tws_client.cpp` so signatures match the actual EWrapper declarations. Do **not** introduce local typedefs; use `int` to stay in sync with the API headers.

### Reference

- TWS API: `EWrapper_prototypes.h` (in the tws-api repo) for the canonical callback signatures.

---

## 3. Live IBKR Test (test_positions_live)

### How to run

From **repo root** (so `config/tws_config.json` is found if present):

```bash
cd /path/to/ib_box_spread_full_universal
export LD_LIBRARY_PATH="${PWD}/build/tws_api_vendor_build/lib:${LD_LIBRARY_PATH}"
./build/bin/test_positions_live
```

### Prerequisites

- TWS or IB Gateway running with API enabled (paper 7497 or live 7496).
- If using a vendored TWS build, the shared library must be in `build/tws_api_vendor_build/lib` (or adjust `LD_LIBRARY_PATH` on Linux; on macOS use `DYLD_LIBRARY_PATH` if needed).
- Optional: `config/tws_config.json` for host/port/client_id; otherwise defaults (127.0.0.1, 7497, 1) are used and the client may auto-switch to an open port (e.g. 7496).

### Notes

- The script `scripts/test_positions_live.sh` targets a different layout (`native/build_native/bin`, port 4001, macOS). For the standard CMake build, use the commands above.
- Build the binary first: `ninja -C build test_positions_live`.

---

## 4. QuantLib Build Failures with Strict Warnings

### Symptom

Build fails when compiling QuantLib (FetchContent) with errors or warnings treated as errors (e.g. unused parameter, shadow, conversion, old-style-cast) coming from QuantLib sources.

### Cause

The top-level `add_compile_options(-Wall -Wextra -Wunused -Wconversion ...)` apply to all targets, including FetchContent subprojects. QuantLib (even v1.41) has code that triggers these warnings.

### Fix (what helped)

1. **Bump QuantLib** to a recent tag (e.g. **v1.41**) in `native/CMakeLists.txt` (`GIT_TAG` in `fetchcontent_declare(QuantLib ...)`).
2. **Per-target suppression for the QuantLib target only** after `fetchcontent_makeavailable(QuantLib)`: detect the actual target name (`ql_library`, `ql`, or `QuantLib`) and add:
   - `-Wno-unused-parameter`
   - `-Wno-unused-variable`
   - `-Wno-shadow`
   - `-Wno-conversion`
   - `-Wno-sign-conversion`
   - `-Wno-old-style-cast`
   - `-Wno-overloaded-virtual`
   so only the QuantLib dependency is relaxed; project code keeps strict flags.

### Code location

- `native/CMakeLists.txt`: QuantLib `fetchcontent_declare` (GIT_TAG), then after `fetchcontent_makeavailable` the block that sets `target_compile_options` on the ql target.

---

## 5. TWS Test Executables Missing Proto Headers

### Symptom

Build fails when compiling `test_tws_connection`, `test_positions_live`, etc., with:
- `ExecutionDetails.pb.h: No such file or directory` (or similar `.pb.h`).

### Cause

When `TWS_API_BUILD_VENDOR` is ON, generated protobuf headers live under `TWS_API_EXTERNAL_BINARY_DIR/proto_generated`. The main app gets this include path and a dependency on `twsapi_external`; the TWS test executables did not.

### Fix (what helped)

For each TWS test executable (`test_tws_connection`, `test_positions_live`, `test_simple_connect`, `test_diagnostic_connect`, `test_packet_trace`), when `TWS_API_BUILD_VENDOR` is set:

- Add `target_include_directories(<target> PRIVATE "${TWS_API_EXTERNAL_BINARY_DIR}/proto_generated")`
- Add `add_dependencies(<target> twsapi_external)`

### Code location

- `native/CMakeLists.txt`: each block that defines one of the TWS test executables.

---

## Summary for AI

| Issue | Root cause | Fix |
|-------|------------|-----|
| Reconnection crash | Replacing `reader_thread_` while old thread still running | Join old reader under mutex before starting new one; signal from `connectionClosed()` |
| OrderId/TickerId errors | API uses `int` in EWrapper | Use `int` in all EWrapper overrides in tws_client.cpp |
| Live test | Binary/config/path | Run from repo root; set LD_LIBRARY_PATH; optional config/tws_config.json |
| QuantLib build | Global strict flags applied to FetchContent | Bump QuantLib; add -Wno-* for ql target only |
| Test binaries missing .pb.h | No proto_generated include/dep for test targets | Add include dir and twsapi_external dependency when TWS_API_BUILD_VENDOR |
