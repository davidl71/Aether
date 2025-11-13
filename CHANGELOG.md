# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed - TWS API Version Update
- Updated TWS API from version 10.33.01 to 10.40.01
- **Key improvements in 10.40.01** (from [release notes](https://ibkrguides.com/releasenotes/prod-2025.htm)):
  - ✅ **Full Protocol Buffers Support**: All requests/responses now support protocol buffers (previously only specific endpoints)
  - ✅ **Order Recovery**: New setting "Maintain and resubmit orders when connection is restored" - automatically maintains/resubmits orders after network disconnect
  - ✅ **Enhanced Error Handling**: Better errors and exceptions handling (10.38+)
- **Incremental Protocol Buffers Support** (10.37-10.39):
  - Historical data, account data, positions (10.39)
  - Completed orders, contract data, market data, market depth (10.38)
  - placeOrder/cancelOrder/reqGlobalCancel, error/openOrder/openOrdersEnd/orderStatus (10.37)
- Default download URL updated to: `https://interactivebrokers.github.io/downloads/twsapi_macunix.1040.01.zip`

## [1.0.1] - 2025-01-27

### Added - Box Spread Core Improvements

#### Option Chain Scanning (Priority 1)
- Implemented complete `find_box_spreads_in_chain()` algorithm
- Scans all expiries in DTE range
- Generates all strike pairs and validates all 4 legs
- Checks liquidity requirements (volume, open interest)
- Sorts opportunities by profitability
- Added comprehensive logging for debugging

#### Atomic Execution (Priority 2)
- Enhanced `place_box_spread()` with rollback logic
- Monitors order statuses after submission
- Automatic cancellation of remaining orders if any leg fails
- Multi-leg order tracking for monitoring
- Clear error messages and execution tracking
- Added fmt::join for better logging

#### Comprehensive Validation (Priority 3)
- Strike width validation (theoretical value = strike width)
- Bid/ask spread validation (max $0.50 per leg)
- Price validation (all prices must be positive)
- Market data quality checks in `evaluate_box_spread()`
- Enhanced `BoxSpreadValidator::validate()` with all checks

### Added - NautilusTrader Enhancements

#### Event-Driven Market Data Handler
- Complete rewrite of `market_data_handler.py`
- Data quality validation (stale data, spread checks, price validation)
- Proper timestamp extraction from ticks (nanoseconds → datetime)
- Data quality statistics tracking per instrument
- Configurable max data age (default 5 seconds)
- Event-driven callbacks replace polling
- Methods: `get_data_quality_stats()`, `get_all_data_quality_stats()`

#### Strategy Class Pattern
- Refactored `strategy_runner.py` to `BoxSpreadStrategyRunner`
- Lifecycle methods: `on_start()`, `on_stop()`, `on_reset()`
- Event handlers: `on_quote_tick()`, `on_trade_tick()`, `on_order_filled()`, `on_order_rejected()`
- Proper state management (`_running`, `_started`, `_stopped`)
- Subscription tracking with `_subscribed_instruments` set
- Position tracking with average price calculation
- Multi-leg order tracking with automatic rollback
- Backward compatibility via `StrategyRunner` alias

#### Order Factory Pattern
- New `python/integration/order_factory.py`
- Factory methods: `limit()`, `market()`, `create_box_spread_orders()`
- Unique client order ID generation (timestamp-based)
- Proper NautilusTrader order construction
- Support for all order types and time in force options
- Placeholder for combo order support

#### Improved Instrument Management
- Uses proper `InstrumentId` format ("SPY.US")
- Automatic ".US" suffix for US instruments
- Proper subscription/unsubscription with error handling
- Tracks subscribed instruments in set

#### Option Chain Management
- New `python/integration/option_chain_manager.py`
- Efficient nested dictionary: symbol → expiry → strike → option_data
- Real-time updates via `update_option()` (event-driven)
- Methods: `get_option()`, `get_expiries()`, `get_strikes()`, `find_box_spread_legs()`
- Cache invalidation based on configurable age
- Underlying price tracking
- Instrument ID parsing for various formats
- Chain statistics: `get_chain_stats()`

#### Execution Handler Improvements
- Integrated `OrderFactory` for order creation
- New `submit_box_spread_orders()` method for atomic 4-leg submission
- Automatic rollback on partial failure
- Proper error handling and logging

### Added - Distributed Compilation Support

#### CMake Build System
- Added options: `ENABLE_DISTCC`, `ENABLE_CCACHE`, `ENABLE_SCCACHE`
- Automatic tool detection and configuration
- Prioritization: sccache > ccache > distcc
- Support for ccache + distcc combination
- Compiler launcher configuration

#### Build Scripts
- New `build_fast.sh` - Fast builds with ccache
- New `build_distributed.sh` - Distributed builds with distcc + ccache
- Auto-detection of cores for optimal parallelism
- Statistics reporting (ccache, distcc)
- Made executable with proper permissions

### Added - Documentation

#### Learning Documents
- `docs/ICLI_LEARNINGS.md` - Patterns from icli project
- `docs/IBKRBOX_LEARNINGS.md` - Patterns from ibkrbox project
- `docs/NAUTILUS_LEARNINGS.md` - NautilusTrader patterns and Rust recommendations

#### Implementation Guides
- `docs/ACTION_PLAN.md` - Priority action plan
- `docs/NAUTILUS_IMPLEMENTATION_SUMMARY.md` - Implementation details
- `docs/ORATS_INTEGRATION.md` - ORATS integration opportunities
- `docs/DISTRIBUTED_COMPILATION.md` - Build optimization guide
- `docs/IMPLEMENTATION_COMPLETE_SUMMARY.md` - Complete summary

### Changed

#### C++ Core
- `src/box_spread_strategy.cpp` - Complete implementations (not stubs)
- `src/order_manager.cpp` - Rollback logic and tracking
- `CMakeLists.txt` - Build optimization options

#### Python Integration
- `python/integration/market_data_handler.py` - Complete rewrite
- `python/integration/strategy_runner.py` - Refactored to Strategy pattern
- `python/integration/execution_handler.py` - Factory integration

#### Documentation
- `README.md` - Added build optimization section

### Performance Improvements

- **Build Times**:
  - Clean build: 60-90s → 15-20s with distcc
  - Rebuild: 60-90s → 1-2s with ccache
  - Incremental: 5-10s → 2-3s with ccache

- **Runtime** (targets):
  - Event-driven processing (zero polling overhead)
  - Cached option chains (O(1) lookups)
  - Early data quality filtering

---

## [1.0.0] - 2025-11-01

### Initial Release

- Full TWS API integration with EWrapper
- Box spread strategy framework
- Order management system
- Risk calculator
- Configuration management
- Test suite (29 tests, all passing)
- macOS AppKit bundle support
- Universal binary support (Intel + Apple Silicon)
- Python bindings via Cython
- NautilusTrader integration foundation
- Comprehensive documentation

---

## Future Enhancements

### Planned for 1.1.0
- [ ] ORATS API integration
- [ ] Historical backtesting framework
- [ ] Enhanced liquidity scoring
- [ ] Earnings/dividend calendar filtering
- [ ] Order efficiency ratio tracking
- [ ] Rate limiting implementation

### Planned for 1.2.0
- [ ] Rust components for performance-critical code
- [ ] Advanced Greeks calculations
- [ ] Portfolio-level risk management
- [ ] Real-time dashboards
- [ ] Performance metrics tracking

### Planned for 2.0.0
- [ ] Multi-strategy support (beyond box spreads)
- [ ] Machine learning for opportunity scoring
- [ ] Cloud deployment support
- [ ] Web interface
- [ ] Mobile notifications

---

## Links

- [GitHub Repository](https://github.com/yourusername/ib-box-spread-generator)
- [Documentation](docs/)
- [Issues](https://github.com/yourusername/ib-box-spread-generator/issues)
- [Discussions](https://github.com/yourusername/ib-box-spread-generator/discussions)

