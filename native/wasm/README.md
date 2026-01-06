# WASM Module for Web App

This directory contains the WebAssembly build configuration for compiling C++ business logic to WASM for use in the React web app.

## Overview

The WASM module compiles pure calculation functions from the C++ codebase, enabling:
- **Code Reuse**: Same calculation logic in backend, TUI, and web
- **Performance**: Near-native speed in the browser
- **Consistency**: Single source of truth for business logic

## Building

### Prerequisites

1. **Install Emscripten SDK**:
```bash
git clone https://github.com/emscripten-core/emsdk.git
cd emsdk
./emsdk install latest
./emsdk activate latest
source ./emsdk_env.sh
```

2. **Verify Installation**:
```bash
emcc --version
```

### Build Command

```bash
# From project root
./scripts/build_wasm.sh
```

Or manually:
```bash
cd native/wasm
mkdir -p build && cd build
emcmake cmake ..
cmake --build . --config Release
```

### Output

The build produces:
- `box_spread_wasm.js` - JavaScript loader
- `box_spread_wasm.wasm` - WebAssembly binary

These are automatically copied to `web/public/wasm/` for use in the React app.

## Architecture

### Modules Compiled to WASM

- `box_spread_strategy.cpp` - Box spread calculations
- `risk_calculator.cpp` - Risk analysis
- `option_chain.cpp` - Greeks calculations

### Excluded Modules

These have dependencies that aren't available in WASM:
- `tws_client.cpp` - TWS API (network)
- `order_manager.cpp` - Depends on TWS
- `tui_app.cpp` - UI rendering
- `pcap_capture.cpp` - Network capture

## Usage in React

See `web/src/hooks/useWasm.ts` for React integration:

```typescript
import { useWasm } from './hooks/useWasm';

function MyComponent() {
  const { ready, calculateBoxSpread } = useWasm();

  if (!ready) return <div>Loading...</div>;

  const result = calculateBoxSpread({
    longCallStrike: 100,
    // ... other fields
  });
}
```

## Development

### Adding New Functions

1. Add function to `wasm_bindings.cpp`
2. Add TypeScript types to `web/src/wasm/types.ts`
3. Export in `EMSCRIPTEN_BINDINGS`
4. Rebuild: `./scripts/build_wasm.sh`

### Testing

WASM functions can be tested in:
- C++ unit tests (shared with native code)
- JavaScript/TypeScript tests in `web/src/wasm/__tests__/`

## Troubleshooting

### Emscripten Not Found
```bash
source /path/to/emsdk/emsdk_env.sh
```

### Build Fails
- Check that all dependencies are WASM-compatible
- Remove or stub logging (spdlog)
- Avoid std::filesystem, std::thread

### Runtime Errors
- Check browser console for WASM errors
- Verify WASM files are in `web/public/wasm/`
- Check that `initWasm()` was called before use

## See Also

- [WASM Integration Plan](../../docs/WASM_INTEGRATION_PLAN.md) - Full implementation plan
- [Web App README](../../web/README.md) - Web app documentation
