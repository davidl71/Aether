# API Documentation Index

This file serves as a reference for all external APIs and libraries used in this project. Use `@docs API_DOCUMENTATION_INDEX.md` in Cursor to give the AI context about these APIs.

## Core Trading APIs

### Interactive Brokers TWS API
- **Official Docs**: https://interactivebrokers.github.io/tws-api/
- **GitHub**: https://github.com/InteractiveBrokers/tws-api
- **Version**: 10.40.01+
- **Key Classes**:
  - `EClient` / `EClientSocket`: Client connection to TWS/Gateway
  - `EWrapper`: Callback interface (93+ methods)
  - `DefaultEWrapper`: Base implementation with default stubs
  - `Contract`: Security definition (stock, option, etc.)
  - `Order`: Order details (price, quantity, type)
  - `OrderState`: Order status and fills
- **Ports**:
  - `7497`: Paper Trading (safe for testing)
  - `7496`: Live Trading (real money!)
- **Location**: `native/third_party/tws-api/IBJts/source/cppclient/client/`
- **Headers**: `native/include/ib_box_spread/tws_client.h`

### Intel Decimal Floating-Point Math Library
- **Official Docs**: https://www.intel.com/content/www/us/en/developer/articles/tool/intel-decimal-floating-point-math-library.html
- **Version**: 20U2
- **Purpose**: Precision decimal arithmetic for financial calculations
- **Key Functions**: `___bid64_add`, `___bid64_div`, `___bid64_mul`, etc.
- **Location**: `native/third_party/IntelRDFPMathLib20U2/LIBRARY/libbid.a`
- **Note**: Required by TWS API for decimal price handling

### Protocol Buffers
- **Official Docs**: https://protobuf.dev/
- **C++ API**: https://protobuf.dev/cpp/
- **Version**: 6.33.0+
- **Purpose**: Serialization for TWS API messages
- **Generated Files**: `*.pb.cc`, `*.pb.h` in TWS API client directory
- **Location**: `/usr/local/lib/libprotobuf.dylib`

## Logging & Utilities

### spdlog
- **Official Docs**: https://github.com/gabime/spdlog
- **API Reference**: https://spdlog.docsforge.com/
- **Version**: Latest (via CMake FetchContent)
- **Usage**: `spdlog::info()`, `spdlog::error()`, `spdlog::warn()`, etc.
- **Key Features**: Fast, header-only, async logging, multiple sinks
- **Example**:
  ```cpp
  #include <spdlog/spdlog.h>
  spdlog::info("Connection established");
  spdlog::error("Failed to connect: {}", error_msg);
  ```

### CLI11
- **Official Docs**: https://cliutils.github.io/CLI11/book/
- **GitHub**: https://github.com/CLIUtils/CLI11
- **Version**: Latest (via CMake FetchContent)
- **Purpose**: Command-line argument parsing
- **Usage**: See `native/src/ib_box_spread.cpp` for examples

### nlohmann/json
- **Official Docs**: https://json.nlohmann.me/
- **GitHub**: https://github.com/nlohmann/json
- **Version**: Latest (via CMake FetchContent)
- **Purpose**: JSON parsing and serialization
- **Usage**: `json::parse()`, `json::dump()`, etc.

## Testing

### Catch2
- **Official Docs**: https://github.com/catchorg/Catch2
- **Documentation**: https://github.com/catchorg/Catch2/blob/devel/docs/Readme.md
- **Version**: Latest (via CMake FetchContent)
- **Usage**: `TEST_CASE()`, `REQUIRE()`, `CHECK()`, etc.
- **Location**: Tests in `native/tests/`

## Build System

### CMake
- **Official Docs**: https://cmake.org/documentation/
- **CMake Tutorial**: https://cmake.org/cmake/help/latest/guide/tutorial/index.html
- **Version**: 3.21+
- **Key Files**:
  - `CMakeLists.txt`: Main build configuration
  - `CMakePresets.json`: Build presets
  - `native/CMakeLists.txt`: Native code build
- **Presets**:
  - `macos-universal-debug`: Development build
  - `macos-universal-release`: Production build

### Abseil (Google C++ Libraries)
- **Official Docs**: https://abseil.io/docs/cpp/
- **GitHub**: https://github.com/abseil/abseil-cpp
- **Version**: 20250814+
- **Purpose**: Required dependency of Protocol Buffers
- **Location**: `/usr/local/lib/libabsl*.dylib`
- **Note**: 184+ individual libraries linked

## Python Integration

### Cython
- **Official Docs**: https://cython.readthedocs.io/
- **Purpose**: Python bindings for C++ code
- **Location**: `python/bindings/`
- **Build**: `cmake --build build --target python_bindings`

### Nautilus Trader
- **Official Docs**: https://docs.nautilustrader.io/
- **GitHub**: https://github.com/nautechsystems/nautilus_trader
- **Version**: 1.221.0+
- **Purpose**: High-performance Python trading framework
- **Location**: `native/third_party/nautilus/`
- **Note**: Optional integration, Python wheel file

## Rust (Agents)

### Rust Standard Library
- **Official Docs**: https://doc.rust-lang.org/std/
- **Location**: `agents/backend/`, `agents/backend-mock/`, etc.
- **Cargo**: `Cargo.toml` files in agent directories

## Go (TUI)

### Go Standard Library
- **Official Docs**: https://pkg.go.dev/std
- **Location**: `tui/`
- **Modules**: `go.mod`, `go.sum`

## TypeScript/JavaScript (Web)

### TypeScript
- **Official Docs**: https://www.typescriptlang.org/docs/
- **Location**: `web/`
- **Config**: `tsconfig.json`

### Vite
- **Official Docs**: https://vitejs.dev/
- **Config**: `vite.config.ts`

## Swift (Desktop/iOS)

### Swift Package Manager
- **Official Docs**: https://www.swift.org/package-manager/
- **Location**: `desktop/`, `ios/`
- **Config**: `Package.swift`

## How to Use This Index in Cursor

### Method 1: Reference in Prompts
When asking Cursor about API usage, reference this file:
```
@docs API_DOCUMENTATION_INDEX.md How do I use spdlog for error logging?
```

### Method 2: Add to Code Comments
Add references in your code:
```cpp
// @docs API_DOCUMENTATION_INDEX.md - TWS API EWrapper implementation
class MyTWSClient : public EWrapper {
  // ...
};
```

### Method 3: Update .cursorrules
The `.cursorrules` file already references this documentation structure.

## Keeping This Index Updated

When adding new dependencies:
1. Add entry to this file with:
   - Official documentation URL
   - Version used
   - Key classes/functions
   - Location in codebase
2. Update version numbers when upgrading
3. Add usage examples for complex APIs

## Quick Reference Links

- **TWS API**: https://interactivebrokers.github.io/tws-api/
- **spdlog**: https://github.com/gabime/spdlog
- **CMake**: https://cmake.org/documentation/
- **Protocol Buffers**: https://protobuf.dev/
- **Catch2**: https://github.com/catchorg/Catch2
- **CLI11**: https://cliutils.github.io/CLI11/book/
- **nlohmann/json**: https://json.nlohmann.me/
- **Nautilus Trader**: https://docs.nautilustrader.io/
