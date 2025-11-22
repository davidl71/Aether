# WASM Quick Start Guide

Quick guide to get started with WASM integration for code reuse.

## What is This?

The WASM module compiles C++ business logic (box spread calculations, risk analysis) to WebAssembly, allowing the React web app to use the same calculation code as the C++ backend and TUI.

## Benefits

1. **Single Source of Truth**: Update calculations once, use everywhere
2. **Consistency**: Same results in backend, TUI, and web
3. **Performance**: Near-native speed in browser
4. **Code Reuse**: Share logic between C++, TUI, and web

## Quick Setup

### 1. Install Emscripten

```bash
git clone https://github.com/emscripten-core/emsdk.git
cd emsdk
./emsdk install latest
./emsdk activate latest
source ./emsdk_env.sh
```

### 2. Build WASM Module

```bash
./scripts/build_wasm.sh
```

This creates:

- `web/public/wasm/box_spread_wasm.js`
- `web/public/wasm/box_spread_wasm.wasm`

### 3. Use in React

```typescript
import { useWasm } from './hooks/useWasm';

function MyComponent() {
  const { ready, calculateBoxSpread, error } = useWasm();

  if (!ready) {
    return <div>Loading calculation engine...</div>;
  }

  if (error) {
    return <div>Error: {error}</div>;
  }

  const result = calculateBoxSpread({
    longCallStrike: 100,
    longCallBid: 5.0,
    longCallAsk: 5.5,
    shortCallStrike: 105,
    shortCallBid: 3.0,
    shortCallAsk: 3.5,
    longPutStrike: 105,
    longPutBid: 3.0,
    longPutAsk: 3.5,
    shortPutStrike: 100,
    shortPutBid: 5.0,
    shortPutAsk: 5.5,
    underlyingPrice: 102.5,
    riskFreeRate: 0.05,
    daysToExpiry: 30,
  });

  return (
    <div>
      <p>Profit: ${result.arbitrageProfit.toFixed(2)}</p>
      <p>ROI: {result.roi.toFixed(2)}%</p>
      <p>APR: {result.apr.toFixed(2)}%</p>
      <p>Profitable: {result.isProfitable ? 'Yes' : 'No'}</p>
    </div>
  );
}
```

## What Gets Compiled?

### ✅ Included (Pure Calculations)

- Box spread calculations
- Risk analysis
- Greeks calculations (delta, gamma, theta, vega)
- Data conversion utilities

### ❌ Excluded (Has Dependencies)

- TWS API client (network)
- Order management (depends on TWS)
- UI rendering
- File I/O

## Next Steps

1. See [WASM Integration Plan](./WASM_INTEGRATION_PLAN.md) for full details
2. Check `native/wasm/README.md` for build details
3. Review `web/src/hooks/useWasm.ts` for React integration

## Troubleshooting

**Emscripten not found:**

```bash
source /path/to/emsdk/emsdk_env.sh
```

**Build fails:**

- Check that Emscripten is activated
- Verify CMake can find Emscripten
- Check `native/wasm/CMakeLists.txt` for configuration

**WASM not loading in browser:**

- Check browser console for errors
- Verify files are in `web/public/wasm/`
- Check that `initWasm()` is called before use
