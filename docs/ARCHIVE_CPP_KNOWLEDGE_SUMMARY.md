# C++ Era Knowledge Archive — Summary

**Archived**: 2026-03-19
**Era**: Before Rust migration; C++ removed from build
**Location**: `docs/archive/`

> The following documents captured the C++ implementation phase (2024-2025) and are preserved here for historical reference. The project is now Rust-first. Key knowledge has been extracted below.

---

## Key Learnings Extracted

### 1. IBKR TWS API Path (Historical)
**Was**: `native/third_party/tws-api/`
**Now**: `../tws-api/` (sibling repository, per AGENTS.md)

The C++ implementation extracted the IBKR TWS API zip to `native/third_party/tws-api/`. The CMake build linked against `libTwsApiCpp.dylib`. This path is now obsolete.

### 2. CMake FetchContent Dependencies
All third-party C++ libs were fetched via CMake `FetchContent`:
```cmake
FetchContent_Declare(Eigen3 GIT_REPOSITORY ... GIT_TAG 3.4.0 GIT_SHALLOW TRUE)
FetchContent_MakeAvailable(Eigen3)
```
This pattern was used for: **Eigen 3.4.0**, **NLopt**, **QuantLib**, **spdlog**, **nlohmann/json**, **CLI11**, **Catch2**.

### 3. Eigen3 — Linear Algebra
**Version**: 3.4.0, header-only, MPL2 license
**Key use cases identified** (unimplemented in Rust):
- Portfolio covariance matrix operations
- Convexity optimization (barbell strategy)
- Portfolio variance: `w^T * C * w`
- QR decomposition for linear systems
- Correlation-to-covariance conversion: `diag(σ) * R * diag(σ)`
**Rust alternative**: `ndarray`, `nalgebra`, `eigen` crate

### 4. NLopt — Nonlinear Optimization
**Status**: Documentation only, never integrated
**Use cases identified** (all unimplemented):
1. Convexity optimization (barbell bond allocation)
2. Portfolio rebalancing (minimize transaction costs)
3. Spare cash allocation (box spreads vs T-bills vs bonds)
4. Risk-constrained optimization (Greeks limits)
**Note**: NLopt is self-contained (no Boost required), FetchContent integration straightforward
**Rust alternative**: `nlopt` crate, `rust-minlp`, or hand-rolled optimization

### 5. QuantLib — Quantitative Finance
**Status**: Documentation only, never integrated (requires Boost)
**Use cases identified**:
- Enhanced option pricing (European, American, exotic options)
- Greeks calculations (Delta, Gamma, Vega, Theta, Rho)
- Volatility surface modeling, implied volatility
- Yield curve construction for risk-free rate estimation
- VaR calculations
**Note**: QuantLib requires Boost (date_time, filesystem, system); heavy dependency
**Rust alternative**: `struments` (QuantLib bindings via Rust), or hand-rolled Black-Scholes + custom Greeks in `quant` crate

### 6. Protobuf Migration (TWS API 10.40.01+)
**Status**: Researched, never migrated
- TWS API supports dual-mode: classic callbacks AND protobuf callbacks
- `tickPriceProtoBuf()`, `orderStatusProtoBuf()` alongside classic versions
- Migration was **optional**; classic API continues to work
- Potential perf benefit from efficient binary serialization
**Note**: Rust `ib_adapter` uses the classic socket API; protobuf not used

### 7. WASM Integration (Abandoned)
**Goal**: Compile C++ business logic to WASM for React web app code reuse
**Modules identified for WASM** (pure calc, no I/O dependencies):
- `box_spread_strategy.cpp` — arbitrage profit, ROI, confidence score
- `risk_calculator.cpp` — VaR, position sizing
- `rate_calculator.cpp` — implied rate calculations
**Architecture**: C++ shared between TUI + WASM module, loaded by React via JS/TS wrapper
**Status**: Architecture designed, never implemented; project migrated to Rust instead

### 8. Distributed Compilation
**Tools evaluated**:
- `distcc` — distribute compilation across network machines
- `ccache` — local cache (10-100x speedup)
- `sccache` — Rust-based distcc+ccache combo (now used in Rust builds)
- `icecream` — alternative to distcc with smart scheduling
**Best**: `ccache` + `distcc` combined
**Note**: Rust `sccache` is now used; C++ distcc setup abandoned

### 9. LEAN Engine Research (Never Integrated)
QuantConnect LEAN was researched as a potential engine:
- Multi-broker support (IBKR, Alpaca, etc.)
- Python/C# algorithm execution
- Cython bindings to connect LEAN to C++ calculations
- LEAN REST API wrapper for PWA/TUI integration
**Status**: Researched extensively, decided against integration; project built own Rust engine instead
**Key insight**: LEAN's broker adapters are well-designed; patterns useful for `ib_adapter` reference

### 10. Cross-Platform Build Notes
- macOS/Windows/Linux all required platform-specific TWS API extraction
- CMake toolchain files for each platform
- Universal binary support (ARM64 + x86_64) on macOS via CMake
- Windows: Visual Studio 2019+ with C++ Desktop workload
- Linux: GCC/Clang with POSIX compatibility

---

## Archived Documents

| File | Description |
|------|-------------|
| `WINDOWS_SETUP_GUIDE.md` | Windows TWS API + C++ build setup (446 lines) |
| `IMPLEMENTATION_GUIDE.md` | TWS API integration walkthrough (720 lines) |
| `QUICK_START.md` | Pre-C++ quick start (obsolete) |
| `CROSS_PLATFORM_SETUP.md` | Multi-platform C++ build |
| `QUICK_START_CROSS_PLATFORM.md` | Cross-platform quick start |
| `DISTRIBUTED_COMPILATION.md` | distcc/ccache/sccache setup (914 lines) |
| `EIGEN_INTEGRATION.md` | Eigen3 linear algebra integration (246 lines) |
| `NLOPT_INTEGRATION_GUIDE.md` | NLopt optimization library (491 lines) |
| `QUANTLIB_INTEGRATION_GUIDE.md` | QuantLib quantitative finance (411 lines) |
| `PROTOBUF_MIGRATION_PLAN.md` | TWS API protobuf migration (431 lines) |
| `WASM_INTEGRATION_PLAN.md` | C++→WASM for web app (723 lines) |
| `WASM_QUICK_START.md` | Emscripten/WASM quick start |
| `EMSCRIPTEN_SETUP.md` | Emscripten toolchain install |
| `LEAN_SETUP.md` | QuantConnect LEAN setup (395 lines) |
| `LEAN_IBKR_SETUP.md` | LEAN + IBKR configuration |
| `LEAN_ALPACA_SETUP.md` | LEAN + Alpaca configuration |
| `LEAN_BROKER_ADAPTERS.md` | LEAN broker adapter capabilities |
| `LEAN_REST_API_WRAPPER_DESIGN.md` | LEAN REST wrapper architecture |
| `LEAN_PWA_TUI_INTEGRATION.md` | LEAN → PWA/TUI integration |
| `LEAN_PWA_TUI_INTEGRATION_ANALYSIS.md` | LEAN feasibility for PWA/TUI |
| `LEAN_PYBIND11_INTEGRATION_ANALYSIS.md` | pybind11 bridge analysis |
| `LEAN_TESTING.md` | LEAN box spread testing |
| `ONIXS_DIRECTCONNECT.md` | OnixS low-latency SDK research |

---

## Relevance to Current Rust Implementation

**High relevance** (reusable knowledge):
- Eigen matrix formulas (covariance, portfolio variance) → `quant` crate
- NLopt optimization use cases (convexity, rebalancing) → `quant` or `risk` crate
- QuantLib reference for option pricing → `quant` crate validation
- TWS API protobuf dual-mode → future `ib_adapter` enhancement
- LEAN broker adapter patterns → `ib_adapter` reference

**Low relevance** (historical only):
- Windows C++ build setup → entirely Rust now
- Emscripten/WASM → Rust WASM if needed
- Cython bindings → not applicable
- Distributed C++ compilation → sccache handles Rust caching
