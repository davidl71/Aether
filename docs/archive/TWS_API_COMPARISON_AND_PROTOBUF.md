# TWS API: Sample App vs ib_box_spread Client, and Protobuf

This document compares the official TWS API C++ sample (TestCppClient) with the ib_box_spread TWS client and tests, and summarizes Protocol Buffers usage in the TWS API (from the tws-api repo’s `source/ProtoBuf_readme.txt`).

---

## Part 1 – TWS API Protobuf (from tws-api README)

Protocol Buffers in the TWS API are used for wire/serialization of many message types. The C++ client headers (e.g. `EWrapper.h`) include generated `.pb.h` files (e.g. `ExecutionDetails.pb.h`).

### Version and responsibility

- Proto files are generated with specific `protoc` versions; if your installed compiler version does not match, you must regenerate the files.
- Version support: <https://protobuf.dev/support/version-support/>

### Per-language notes

| Language | Protobuf version (from readme) | Generate from `source/` |
|----------|-------------------------------|--------------------------|
| **Java** | 4.29.5 (jar in `source/javaclient/jars`) | `protoc --proto_path=./proto --java_out=./javaclient proto/*.proto` |
| **C++ (Windows)** | 5.29.5 (vcpkg) | `protoc --proto_path=./proto --cpp_out=./cppclient/client/protobuf proto/*.proto` |
| **C++ (Linux)** | 3.12.4 | `protoc --proto_path=./proto --experimental_allow_proto3_optional --cpp_out=./cppclient/client/protobufUnix proto/*.proto` |
| **C#** | 3.29.5 (NuGet Google.Protobuf) | `protoc --proto_path=./proto --csharp_out=./csharpclient/client/protobuf proto/*.proto` |
| **Python** | 5.29.5 (`pip install protobuf`) | `protoc --proto_path=./proto --python_out=./pythonclient/ibapi/protobuf proto/*.proto` |

### C++ (Windows) – vcpkg

```text
git clone https://github.com/Microsoft/vcpkg.git
cd vcpkg
bootstrap-vcpkg.bat
vcpkg integrate install
vcpkg install protobuf
```

- Protobuf for Visual Studio: <https://vcpkg.io/en/package/protobuf.html>
- vcpkg in VS: <https://devblogs.microsoft.com/cppblog/vcpkg-is-now-included-with-visual-studio/>

### C++ (Linux)

```bash
apt-get install protobuf-compiler
```

- Installation: <https://protobuf.dev/installation/>

### Proto source layout (GitHub tws-api)

- Proto sources: `tws-api/source/proto/*.proto` (e.g. `ExecutionDetails.proto`, `Contract.proto`, `Position.proto`).
- C++ client includes: `EWrapper.h` includes many `*.pb.h` headers; these must be generated (or present in the client tree) for the C++ client to build.

---

## Part 2 – How ib_box_spread uses TWS API and Protobuf

When using the **GitHub tws-api layout** (e.g. `-DTWS_API_SOURCE_DIR=/path/to/tws-api`):

- **Proto source**: `tws-api/source/proto/`.
- **Generation**: Our `native/ibapi_cmake/CMakeLists.txt` uses `find_package(Protobuf)` and generates `.pb.cc` / `.pb.h` into the **build** directory (`proto_generated`), not into the tws-api tree.
- **Include path**: The main app adds `TWS_API_EXTERNAL_BINARY_DIR/proto_generated` so that `#include "ExecutionDetails.pb.h"` (and other `.pb.h`) resolve when building `ib_box_spread` and the TWS API library.
- **protoc flags**: We pass `--proto_path=` pointing at `source/proto` and `--experimental_allow_proto3_optional` for compatibility with the TWS proto3 optionals.

So we do **not** rely on pre-generated `protobuf` / `protobufUnix` in the tws-api repo; we generate at configure/build time and use a single generated output dir for both the TWS API library and the app.

**Recommended:** Use Protobuf 3.12+ (Linux) or 5.x (Windows) per TWS API `source/ProtoBuf_readme.txt` when building with the GitHub tws-api layout. Our CMake uses `find_package(Protobuf)` and `--experimental_allow_proto3_optional`; no version is enforced.

**Config:** Default `connect_options` is `"+PACEAPI"` (matches TWS API sample). You can set `"connect_options": ""` in config to clear it, or set `"optional_capabilities"` under `tws` if needed. `connect_options` and `optional_capabilities` are validated as ASCII printable.

---

## Part 3 – TestCppClient vs ib_box_spread client and tests

### Architecture

| Aspect | TWS API TestCppClient | ib_box_spread TWSClient |
|--------|------------------------|---------------------------|
| **Purpose** | Demo/test harness for many API features | Production wrapper for box-spread strategy |
| **EWrapper base** | `EWrapper` directly | `DefaultEWrapper` (override only what’s needed) |
| **EClient** | `new EClientSocket(this, &m_osSignal)` in ctor | `EClientSocket` + `EReaderOSSignal` inside `Impl` |
| **Threading** | Single-threaded: main loop runs state machine and pumps messages | Dedicated **EReader thread** runs `processMsgs()`; main thread runs strategy and calls `process_messages(timeout)` |
| **Connection “ready”** | After `eConnect()`; no explicit handshake wait | Waits for **connectAck → managedAccounts → nextValidId** before `connect()` returns |
| **Config** | argv: host, port, clientId; optional connectOptions (e.g. `+PACEAPI`) | `config::TWSConfig` (host, port, timeouts, auto_reconnect, mock, etc.) from JSON/CLI |

### Connection and message loop

**TestCppClient**

- `connect(host, port, clientId)`: `m_pClient->eConnect()`, then create `EReader` and `m_pReader->start()`.
- Main loop: `while (client.isConnected()) { client.processMessages(); }`.
- `processMessages()`: large `switch (m_state)` to run one demo (tick data, historical, orders, etc.), then `m_osSignal.waitForSignal()` and `m_pReader->processMsgs()`.
- Single thread both drives the demo state and processes TWS messages.

**ib_box_spread TWSClient**

- `connect()`: tries multiple ports (e.g. 7497, 7496, 4002, 4001), starts EReader in a **background thread** that runs `waitForSignal()` and `processMsgs()`.
- Connection is considered up only after **connectAck**, **managedAccounts**, and **nextValidId** (with timeout).
- Main app calls `process_messages(timeout)`; the reader thread does the actual message pumping.
- Supports mock mode, auto-reconnect with backoff, and rate limiting.

### API surface

**TestCppClient**

- One large EWrapper; state enum drives which request runs next (tick data, historical, orders, scanners, etc.).
- Uses TWS types (`Contract`, `Order`, etc.) and console output; callback-driven only.

**ib_box_spread TWSClient**

- Focused on strategy needs: market data, contract details, orders (including combo), positions, account, margin.
- Sync and async: e.g. `request_market_data_sync()`, `request_positions_sync()`, `request_contract_details_sync()` with timeouts.
- Domain types: `types::OptionContract`, `types::MarketData`, `types::Position`, `types::Order`, etc.
- Callbacks for order status and errors; rate limiter, error guidance, optional PCAP/NATS.

### Tests

**TWS API**

- TestCppClient is a manual sample; no unit or integration test framework in the C++ sample.

**ib_box_spread**

- **test_tws_connection** – standalone: connect with config, print state, keep alive, disconnect.
- **test_positions_live** – connect, stabilize with `process_messages(100)`, request positions sync, print, disconnect.
- **test_simple_connect** – minimal connect + `process_messages(100)` + disconnect.
- **test_diagnostic_connect** – connect and trace connectAck / managedAccounts / nextValidId.
- **test_packet_trace** – optional low-level connection/trace.
- **test_tws_integration.cpp** – Catch2: mock-only connect/disconnect, reconnection, state; optional real-TWS test.

### Summary

| Item | TestCppClient | ib_box_spread client & tests |
|------|----------------|------------------------------|
| **Threading** | Single-thread (state machine + message pump in one loop) | EReader in background thread; main thread for app |
| **Connection ready** | After `eConnect()` succeeds | After connectAck + managedAccounts + nextValidId |
| **Config** | argv | TWSConfig from file + CLI |
| **API style** | Broad demo, callback-only, TWS types | Focused (options, orders, positions), sync + async, domain types |
| **Mock / tests** | None | Mock client, Catch2, standalone connection/position tests |
| **Reconnect** | Manual (outer loop in Main.cpp) | Auto-reconnect with backoff |
| **Rate limiting** | None | Built-in |
| **Logging** | printf | spdlog |

---

## References

- TWS API (GitHub): `tws-api` repo; C++ client under `source/cppclient/client/`; protos under `source/proto/`.
- TWS API Protobuf readme: `tws-api/source/ProtoBuf_readme.txt`.
- ib_box_spread TWS integration: `native/ibapi_cmake/CMakeLists.txt` (proto generation), `native/CMakeLists.txt` (proto_generated include for `ib_box_spread` when `TWS_API_BUILD_VENDOR` is ON), `native/include/tws_client.h`, `native/src/tws_client.cpp`.
