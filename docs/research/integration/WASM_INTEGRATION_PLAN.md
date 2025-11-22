# WASM Integration Plan: C++ Business Logic for Web App

## Overview

This plan outlines how to compile C++ business logic to WebAssembly (WASM) for use in the React web app, enabling code reuse between backend, TUI, and web frontend while reducing code divergence.

## Goals

1. **Code Reuse**: Share calculation logic between C++ backend, TUI, and web app
2. **Reduce Divergence**: Single source of truth for business logic
3. **Performance**: Fast calculations in browser (WASM is near-native speed)
4. **Maintainability**: Update calculations once, use everywhere

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    React Web App                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │   UI Layer   │  │  State Mgmt  │  │   WASM API   │ │
│  │  (React)     │  │  (React)     │  │  (JS/TS)     │ │
│  └──────────────┘  └──────────────┘  └──────┬───────┘ │
│                                              │          │
└──────────────────────────────────────────────┼──────────┘
                                               │
                                    ┌──────────▼──────────┐
                                    │   WASM Module        │
                                    │  (box_spread.wasm)   │
                                    │                      │
                                    │  - Calculations     │
                                    │  - Risk Analysis     │
                                    │  - Data Conversion   │
                                    └──────────────────────┘
                                               │
                                    ┌──────────▼──────────┐
                                    │   C++ Source        │
                                    │  (Shared with TUI)  │
                                    └──────────────────────┘
```

## Modules to Compile to WASM

### ✅ Pure Calculation Modules (No Dependencies)

These modules can be compiled directly to WASM:

1. **`box_spread_strategy.cpp`** (calculation functions only)
   - `calculate_arbitrage_profit()`
   - `calculate_roi()`
   - `calculate_confidence_score()`
   - `is_profitable()`
   - `evaluate_box_spread()`

2. **`risk_calculator.cpp`**
   - `calculate_box_spread_risk()`
   - `calculate_position_risk()`
   - `calculate_portfolio_risk()`
   - VaR calculations
   - Position sizing

3. **`option_chain.cpp`** (Greeks calculations)
   - `calculate_theoretical_price()` (Black-Scholes)
   - `calculate_delta()`
   - `calculate_gamma()`
   - `calculate_theta()`
   - `calculate_vega()`
   - `calculate_implied_volatility()`

4. **`ml_predictor.cpp`** (if XGBoost can be compiled to WASM)
   - `predict_profitability()`
   - `predict_fill_probability()`
   - Feature extraction

5. **`tui_converter.cpp`** (data conversion)
   - Convert between native types and display types
   - JSON serialization helpers

6. **`box_spread_bag.cpp`** (if it has calculations)
   - Bag calculations
   - Multi-leg aggregations

### ❌ Modules NOT for WASM (Have Dependencies)

These require network/system APIs and should stay server-side:

- `tws_client.cpp` - TWS API integration
- `order_manager.cpp` - Depends on TWS client
- `tui_app.cpp` - UI rendering
- `pcap_capture.cpp` - Network capture
- `rate_limiter.cpp` - System time dependencies

## Implementation Plan

### Phase 1: Setup WASM Build System (Week 1-2)

#### 1.1 Install Emscripten

```bash
# Install Emscripten SDK
git clone https://github.com/emscripten-core/emsdk.git
cd emsdk
./emsdk install latest
./emsdk activate latest
source ./emsdk_env.sh
```

#### 1.2 Create WASM Build Directory

```
native/
  wasm/
    CMakeLists.txt          # WASM-specific CMake config
    src/
      wasm_bindings.cpp     # JavaScript bindings
      wasm_exports.cpp      # Exported functions
    include/
      wasm_types.h          # WASM-compatible type definitions
```

#### 1.3 Create WASM CMakeLists.txt

```cmake
# native/wasm/CMakeLists.txt
cmake_minimum_required(VERSION 3.21)
project(ib_box_spread_wasm)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Emscripten settings
set(CMAKE_C_COMPILER emcc)
set(CMAKE_CXX_COMPILER em++)
set(CMAKE_EXECUTABLE_SUFFIX ".js")

# WASM-specific flags
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -O3 -s WASM=1 -s EXPORT_ES6=1")
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -s MODULARIZE=1 -s EXPORT_NAME=createBoxSpreadModule")
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -s ALLOW_MEMORY_GROWTH=1")
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -s EXPORTED_FUNCTIONS='[\"_malloc\",\"_free\"]'")
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -s EXPORTED_RUNTIME_METHODS='[\"ccall\",\"cwrap\",\"UTF8ToString\",\"stringToUTF8\"]'")

# Include directories
include_directories(
  ${CMAKE_CURRENT_SOURCE_DIR}/../include
  ${CMAKE_CURRENT_SOURCE_DIR}/include
)

# Source files (calculation modules only)
set(WASM_SOURCES
  ${CMAKE_CURRENT_SOURCE_DIR}/../src/box_spread_strategy.cpp
  ${CMAKE_CURRENT_SOURCE_DIR}/../src/risk_calculator.cpp
  ${CMAKE_CURRENT_SOURCE_DIR}/../src/option_chain.cpp
  ${CMAKE_CURRENT_SOURCE_DIR}/../src/tui_converter.cpp
  ${CMAKE_CURRENT_SOURCE_DIR}/src/wasm_bindings.cpp
  ${CMAKE_CURRENT_SOURCE_DIR}/src/wasm_exports.cpp
)

# Headers
set(WASM_HEADERS
  ${CMAKE_CURRENT_SOURCE_DIR}/../include/box_spread_strategy.h
  ${CMAKE_CURRENT_SOURCE_DIR}/../include/risk_calculator.h
  ${CMAKE_CURRENT_SOURCE_DIR}/../include/option_chain.h
  ${CMAKE_CURRENT_SOURCE_DIR}/../include/tui_converter.h
  ${CMAKE_CURRENT_SOURCE_DIR}/../include/types.h
  ${CMAKE_CURRENT_SOURCE_DIR}/include/wasm_types.h
)

# Create WASM module
add_executable(box_spread_wasm ${WASM_SOURCES} ${WASM_HEADERS})

# Link libraries (if needed)
# Note: Avoid heavy dependencies like spdlog in WASM
```

### Phase 2: Create WASM-Compatible Wrapper (Week 2-3)

#### 2.1 Create WASM Type Definitions

```cpp
// native/wasm/include/wasm_types.h
#pragma once

#include <cstdint>
#include <cstddef>

// WASM-compatible types (avoid std::string, use char*)
namespace wasm {

// Box spread input (from JavaScript)
struct BoxSpreadInput {
  double long_call_strike;
  double long_call_bid;
  double long_call_ask;
  double short_call_strike;
  double short_call_bid;
  double short_call_ask;
  double long_put_strike;
  double long_put_bid;
  double long_put_ask;
  double short_put_strike;
  double short_put_bid;
  double short_put_ask;
  double underlying_price;
  double risk_free_rate;
  double days_to_expiry;
};

// Box spread result (to JavaScript)
struct BoxSpreadResult {
  double net_debit;
  double arbitrage_profit;
  double roi;
  double apr;
  double confidence_score;
  bool is_profitable;
  double risk_score;
};

} // namespace wasm
```

#### 2.2 Create JavaScript Bindings

```cpp
// native/wasm/src/wasm_bindings.cpp
#include <emscripten/bind.h>
#include "wasm_types.h"
#include "../include/box_spread_strategy.h"
#include "../include/risk_calculator.h"
#include <memory>

using namespace emscripten;

// Wrapper function for box spread calculation
wasm::BoxSpreadResult calculate_box_spread(const wasm::BoxSpreadInput& input) {
  // Convert WASM input to native types
  types::BoxSpreadLeg spread;
  spread.long_call.strike = input.long_call_strike;
  spread.long_call.bid = input.long_call_bid;
  spread.long_call.ask = input.long_call_ask;
  // ... populate all legs

  // Use shared C++ calculation logic
  strategy::BoxSpreadStrategy strategy(/* params */);
  double profit = strategy.calculate_arbitrage_profit(spread);
  double roi = strategy.calculate_roi(spread);
  double confidence = strategy.calculate_confidence_score(spread, /* chain */);

  // Convert back to WASM result
  wasm::BoxSpreadResult result;
  result.arbitrage_profit = profit;
  result.roi = roi;
  result.confidence_score = confidence;
  result.is_profitable = strategy.is_profitable(spread);

  return result;
}

// Risk calculation wrapper
double calculate_risk(
  double position_size,
  double volatility,
  double time_horizon
) {
  // Use shared risk calculator
  risk::RiskCalculator calc(/* config */);
  // ... calculate risk
  return 0.0; // placeholder
}

// Export to JavaScript
EMSCRIPTEN_BINDINGS(box_spread_wasm) {
  // Register types
  value_object<wasm::BoxSpreadInput>("BoxSpreadInput")
    .field("longCallStrike", &wasm::BoxSpreadInput::long_call_strike)
    .field("longCallBid", &wasm::BoxSpreadInput::long_call_bid)
    // ... all fields
    ;

  value_object<wasm::BoxSpreadResult>("BoxSpreadResult")
    .field("netDebit", &wasm::BoxSpreadResult::net_debit)
    .field("arbitrageProfit", &wasm::BoxSpreadResult::arbitrage_profit)
    .field("roi", &wasm::BoxSpreadResult::roi)
    .field("apr", &wasm::BoxSpreadResult::apr)
    .field("confidenceScore", &wasm::BoxSpreadResult::confidence_score)
    .field("isProfitable", &wasm::BoxSpreadResult::is_profitable)
    .field("riskScore", &wasm::BoxSpreadResult::risk_score)
    ;

  register_vector<wasm::BoxSpreadResult>("BoxSpreadResultVector");

  // Export functions
  function("calculateBoxSpread", &calculate_box_spread);
  function("calculateRisk", &calculate_risk);
}
```

### Phase 3: JavaScript/TypeScript Wrapper (Week 3-4)

#### 3.1 Create TypeScript Types

```typescript
// web/src/wasm/types.ts
export interface BoxSpreadInput {
  longCallStrike: number;
  longCallBid: number;
  longCallAsk: number;
  shortCallStrike: number;
  shortCallBid: number;
  shortCallAsk: number;
  longPutStrike: number;
  longPutBid: number;
  longPutAsk: number;
  shortPutStrike: number;
  shortPutBid: number;
  shortPutAsk: number;
  underlyingPrice: number;
  riskFreeRate: number;
  daysToExpiry: number;
}

export interface BoxSpreadResult {
  netDebit: number;
  arbitrageProfit: number;
  roi: number;
  apr: number;
  confidenceScore: number;
  isProfitable: boolean;
  riskScore: number;
}
```

#### 3.2 Create WASM Module Loader

```typescript
// web/src/wasm/loader.ts
import createBoxSpreadModule from '../../wasm/box_spread_wasm.js';

let wasmModule: any = null;
let isInitialized = false;

export async function initWasm(): Promise<void> {
  if (isInitialized) {
    return;
  }

  try {
    wasmModule = await createBoxSpreadModule();
    isInitialized = true;
    console.log('WASM module loaded successfully');
  } catch (error) {
    console.error('Failed to load WASM module:', error);
    throw error;
  }
}

export function isWasmReady(): boolean {
  return isInitialized && wasmModule !== null;
}

export function getWasmModule(): any {
  if (!isWasmReady()) {
    throw new Error('WASM module not initialized. Call initWasm() first.');
  }
  return wasmModule;
}
```

#### 3.3 Create React Hook for WASM

```typescript
// web/src/hooks/useWasm.ts
import { useEffect, useState } from 'react';
import { initWasm, isWasmReady, getWasmModule } from '../wasm/loader';
import type { BoxSpreadInput, BoxSpreadResult } from '../wasm/types';

export function useWasm() {
  const [ready, setReady] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    initWasm()
      .then(() => setReady(true))
      .catch((err) => setError(err.message));
  }, []);

  const calculateBoxSpread = (input: BoxSpreadInput): BoxSpreadResult => {
    if (!isWasmReady()) {
      throw new Error('WASM not ready');
    }

    const module = getWasmModule();
    return module.calculateBoxSpread(input);
  };

  const calculateRisk = (
    positionSize: number,
    volatility: number,
    timeHorizon: number
  ): number => {
    if (!isWasmReady()) {
      throw new Error('WASM not ready');
    }

    const module = getWasmModule();
    return module.calculateRisk(positionSize, volatility, timeHorizon);
  };

  return {
    ready,
    error,
    calculateBoxSpread,
    calculateRisk,
  };
}
```

### Phase 4: Integrate with React Components (Week 4-5)

#### 4.1 Update BoxSpreadTable Component

```typescript
// web/src/components/BoxSpreadTable.tsx (updated)
import { useWasm } from '../hooks/useWasm';
import type { BoxSpreadScenario } from '../types';

function BoxSpreadTable({ scenarios, asOf, underlying }: BoxSpreadTableProps) {
  const { ready: wasmReady, calculateBoxSpread } = useWasm();
  const [enhancedScenarios, setEnhancedScenarios] = useState(scenarios);

  useEffect(() => {
    if (wasmReady && scenarios.length > 0) {
      // Recalculate using WASM for consistency
      const enhanced = scenarios.map((scenario) => {
        const input = {
          longCallStrike: scenario.long_call_strike,
          longCallBid: scenario.long_call_bid,
          longCallAsk: scenario.long_call_ask,
          // ... populate all fields
          underlyingPrice: underlying,
          riskFreeRate: 0.05, // from config
          daysToExpiry: scenario.dte,
        };

        const result = calculateBoxSpread(input);

        return {
          ...scenario,
          // Override with WASM-calculated values
          annualized_return: result.apr,
          fill_probability: result.confidenceScore / 100,
          is_profitable: result.isProfitable,
        };
      });

      setEnhancedScenarios(enhanced);
    }
  }, [wasmReady, scenarios, underlying, calculateBoxSpread]);

  // ... rest of component
}
```

#### 4.2 Add WASM Loading Indicator

```typescript
// web/src/App.tsx (add WASM loading)
import { useWasm } from './hooks/useWasm';

function App() {
  const { ready: wasmReady, error: wasmError } = useWasm();

  // ... existing code

  return (
    <div className="app-shell">
      {!wasmReady && (
        <div className="wasm-loading">
          Loading calculation engine...
        </div>
      )}
      {wasmError && (
        <div className="wasm-error">
          Warning: Calculation engine unavailable. Using server calculations.
        </div>
      )}
      {/* ... rest of app */}
    </div>
  );
}
```

### Phase 5: Build Integration (Week 5-6)

#### 5.1 Update Vite Config

```typescript
// web/vite.config.ts (updated)
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { VitePWA } from 'vite-plugin-pwa';

export default defineConfig({
  plugins: [
    react(),
    VitePWA({
      // ... existing PWA config
      // Add WASM files to precache
      workbox: {
        globPatterns: ['**/*.{js,css,html,wasm}'],
      },
    }),
  ],
  // Copy WASM files to dist
  publicDir: 'public',
  build: {
    rollupOptions: {
      output: {
        // Keep WASM files separate
        assetFileNames: (assetInfo) => {
          if (assetInfo.name?.endsWith('.wasm')) {
            return 'wasm/[name][extname]';
          }
          return 'assets/[name]-[hash][extname]';
        },
      },
    },
  },
  // Optimize WASM loading
  optimizeDeps: {
    exclude: ['box_spread_wasm'],
  },
});
```

#### 5.2 Create Build Script

```bash
#!/bin/bash
# scripts/build_wasm.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
WASM_DIR="${PROJECT_ROOT}/native/wasm"
WEB_WASM_DIR="${PROJECT_ROOT}/web/public/wasm"

# Source Emscripten
if [ -z "$EMSDK" ]; then
  echo "Error: Emscripten not found. Install and activate emsdk first."
  exit 1
fi

# Build WASM
cd "${WASM_DIR}"
mkdir -p build
cd build
cmake ..
cmake --build . --config Release

# Copy to web public directory
mkdir -p "${WEB_WASM_DIR}"
cp box_spread_wasm.js "${WEB_WASM_DIR}/"
cp box_spread_wasm.wasm "${WEB_WASM_DIR}/"

echo "✅ WASM build complete"
echo "   Files copied to: ${WEB_WASM_DIR}"
```

### Phase 6: Testing & Validation (Week 6-7)

#### 6.1 Create WASM Tests

```typescript
// web/src/wasm/__tests__/wasm.test.ts
import { initWasm, getWasmModule } from '../loader';
import type { BoxSpreadInput } from '../types';

describe('WASM Module', () => {
  beforeAll(async () => {
    await initWasm();
  });

  it('should calculate box spread correctly', () => {
    const module = getWasmModule();
    const input: BoxSpreadInput = {
      longCallStrike: 100,
      longCallBid: 5.0,
      longCallAsk: 5.5,
      // ... complete input
    };

    const result = module.calculateBoxSpread(input);

    expect(result).toHaveProperty('arbitrageProfit');
    expect(result).toHaveProperty('roi');
    expect(result).toHaveProperty('apr');
    expect(typeof result.isProfitable).toBe('boolean');
  });

  it('should match server-side calculations', async () => {
    // Compare WASM results with server API results
    // This ensures consistency
  });
});
```

#### 6.2 Performance Testing

```typescript
// web/src/wasm/__tests__/performance.test.ts
describe('WASM Performance', () => {
  it('should calculate 1000 box spreads in < 100ms', () => {
    const module = getWasmModule();
    const start = performance.now();

    for (let i = 0; i < 1000; i++) {
      module.calculateBoxSpread(mockInput);
    }

    const duration = performance.now() - start;
    expect(duration).toBeLessThan(100);
  });
});
```

## Dependencies to Handle

### ✅ Safe for WASM

- Standard library (std::vector, std::string with care)
- Math functions (std::cmath)
- JSON (nlohmann/json - can compile to WASM)

### ⚠️ Needs Adaptation

- **spdlog**: Remove or stub (logging not needed in WASM)
- **std::filesystem**: Not available in WASM, use alternatives
- **std::thread**: Use Emscripten's async APIs
- **std::chrono**: Limited support, use JavaScript Date

### ❌ Not Available in WASM

- Network APIs (use fetch from JavaScript)
- File I/O (use JavaScript APIs)
- System calls

## Code Sharing Strategy

### Shared Code Structure

```
native/
  src/
    box_spread_strategy.cpp    ← Shared (used by TUI + WASM)
    risk_calculator.cpp         ← Shared
    option_chain.cpp            ← Shared
    types.h                     ← Shared
  wasm/
    src/
      wasm_bindings.cpp         ← WASM-only (bindings)
      wasm_exports.cpp          ← WASM-only (exports)
```

### Avoiding Code Duplication

1. **Pure Functions**: Keep calculation functions pure (no side effects)
2. **Type Definitions**: Share `types.h` between TUI and WASM
3. **Build System**: Use CMake to compile same sources for different targets
4. **Testing**: Share unit tests where possible

## Migration Checklist

- [ ] Phase 1: Setup WASM build system
- [ ] Phase 2: Create WASM-compatible wrappers
- [ ] Phase 3: Create JavaScript/TypeScript bindings
- [ ] Phase 4: Integrate with React components
- [ ] Phase 5: Build integration
- [ ] Phase 6: Testing & validation
- [ ] Phase 7: Performance optimization
- [ ] Phase 8: Documentation
- [ ] Phase 9: Deploy and monitor

## Timeline

- **Week 1-2**: Setup and build system
- **Week 3-4**: WASM wrappers and bindings
- **Week 5-6**: React integration
- **Week 7**: Testing and optimization
- **Total**: ~7 weeks for full implementation

## Benefits

1. **Single Source of Truth**: Calculations defined once in C++
2. **Consistency**: Same results in backend, TUI, and web
3. **Performance**: Near-native speed in browser
4. **Maintainability**: Update calculations once, deploy everywhere
5. **Type Safety**: Shared type definitions prevent errors

## Next Steps

1. Review and approve this plan
2. Set up Emscripten development environment
3. Start with Phase 1 (build system)
4. Iterate on one calculation module first (box_spread_strategy)
5. Expand to other modules incrementally
