# Aether - Synthetic Financing Platform

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)]()
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![C++](https://img.shields.io/badge/C%2B%2B-20-blue.svg)]()

Comprehensive multi-asset financing optimization system for managing synthetic financing across options, futures, bonds, bank loans, and pension funds. Provides unified portfolio management, cash flow modeling, opportunity simulation, and multi-instrument relationship optimization across 21+ accounts and multiple brokers.

**Note**: Box spreads are one strategy component of this platform, used for spare cash allocation (7-10% of portfolio) to achieve T-bill-equivalent yields.

## ⚠️ Important Disclaimers

**READ THIS CAREFULLY BEFORE USING**

- **This is trading software**: Real money is at risk. You can lose money. Use entirely at your own risk.
- **Paper trading first**: ALWAYS test thoroughly in paper trading mode before considering live trading.
- **Not financial advice**: This software is for educational and research purposes only.
- **Regulatory compliance**: Ensure you comply with all applicable securities regulations.
- **No warranties**: THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND.
- **Stub implementation**: The current TWS API client is a stub. You must integrate the actual IBKR TWS API.

## Platform Capabilities

### Core Platform Features
- ✅ Multi-account aggregation (21+ accounts across multiple brokers)
- ✅ Cash flow modeling and forecasting
- ✅ Opportunity simulation (what-if analysis)
- ✅ Multi-instrument relationship optimization
- ✅ Investment strategy framework (portfolio allocation, convexity, volatility skew)
- ✅ Unified position view across all asset types
- ✅ Multi-broker architecture (IBKR, Israeli brokers, future specialist integrations)

Tradier support is currently removed from the active platform surface. Reintroduction can be revisited later if it becomes a real integration priority.

Jupyter notebooks remain available as manual research artifacts under `notebooks/`, but the
old project-managed JupyterLab service path is retired for now and should be treated as a
future improvement rather than an active runtime feature.

The React web app under `web/` is also retired as an active runtime surface for now. The
current supported frontends are the Rust TUI and the native CLI. The web tree is kept only as
archived implementation/reference material while the project focuses on TUI/CLI.

### Additional Platform Features
- ✅ Comprehensive logging with spdlog
- ✅ Dry-run mode for safe testing
- ✅ Universal binary support (Intel + Apple Silicon on macOS)
- ✅ Modern C++20 codebase with extensive error handling
- ✅ JSON-based configuration
- ✅ Comprehensive test suite with Catch2
- ✅ QuantConnect-inspired deployment guard rails (pre-flight checklist, weekly re-auth, rich error catalog)
- ✅ Event notifications via email/webhook/SMS/Telegram
- ✅ Market data provider failover with ORATS fallback support
- ✅ QuestDB time-series archiving for quotes and trades
- ✅ IBKR Client Portal API integration for account and portfolio snapshots
- ✅ Cython bindings exposing C++ calculations to Python
- ✅ WebAssembly (WASM) module for code reuse between backend and future UI surfaces

## Strategies

### Box Spread Strategy ⭐ (Active)

**Purpose**: Synthetic financing via options arbitrage
**Allocation**: 7-10% of portfolio (spare cash)
**Use Case**: Enhanced yield on spare cash comparable to T-bills or SOFR

**Features**:
- ✅ Automated box spread identification and analysis
- ✅ Real-time options chain monitoring
- ✅ Risk-based position sizing and management
- ✅ IBKR BAG order execution (atomic 4-leg execution)

**Documentation**: See [Box Spread Strategy Documentation](docs/strategies/box-spread/README.md)

### Futures Strategy (Planned)

**Purpose**: Implied financing rates from futures
**Status**: Design phase

### Bond Strategy (Planned)

**Purpose**: Direct financing via bond ETFs
**Allocation**: 30-40% of portfolio (core investments)
**Status**: Design phase

### Loan Strategy (Planned)

**Purpose**: Secured financing via bank/pension loans
**Status**: Design phase

---

## What is a Box Spread?

A box spread is a complex options strategy that combines four positions:

1. **Long call** at strike K1 (lower strike)
2. **Short call** at strike K2 (higher strike)
3. **Long put** at strike K2 (higher strike)
4. **Short put** at strike K1 (lower strike)

**Synthetic Financing**: Box spreads create synthetic lending/borrowing positions. The implied interest rate is calculated from the difference between strike width and net debit/credit, providing a risk-free financing rate comparable to T-bills or SOFR.

**Example** (Borrowing Scenario):

- Buy SPX 5000 Call @ $250
- Sell SPX 5050 Call @ $100
- Buy SPX 5050 Put @ $200
- Sell SPX 5000 Put @ $75
- **Net Debit**: $275 (cost to establish position)
- **Strike Width**: $50.00
- **Implied Interest Rate**: ((275 - 50) / 50) × (365 / 30) × 100% = 5.48% APR
- **Use Case**: Synthetic borrowing at 5.48% APR, competitive with T-bills or margin loans

## Prerequisites

### System Requirements

- **macOS** 11.0+ / **Linux** / **Windows 10/11** (64-bit)
- **CMake** 3.21 or higher
- **C++ Compiler** with C++20 support:
  - **macOS**: Clang 13+ (recommended)
  - **Linux**: GCC 11+ or Clang 13+
  - **Windows**: Visual Studio 2019+ or MinGW-w64
- **Interactive Brokers** TWS or IB Gateway
- **Active IBKR account** with options trading enabled

**Note:** See [Windows Setup Guide](docs/WINDOWS_SETUP_GUIDE.md) for Windows-specific instructions.

### Dependencies (Automatically Downloaded)

The following dependencies are automatically fetched by CMake:

- [nlohmann/json](https://github.com/nlohmann/json) - JSON parsing
- [spdlog](https://github.com/gabime/spdlog) - Logging
- [CLI11](https://github.com/CLIUtils/CLI11) - Command-line parsing
- [Catch2](https://github.com/catchorg/Catch2) - Testing framework

### Third-Party Bundles (Cached at Build Time)

Heavy vendor assets now live outside the repository and are hydrated locally:

```bash
./scripts/fetch_third_party.sh
```

This script downloads/extracts:

- **Protobuf v3.20.3** (override with `PROTOBUF_URL` if you mirror releases)
- **Intel decimal math library** (provide `INTEL_DECIMAL_URL` or drop the tarball into `native/third_party/cache/`)
- **IBKR TWS API** (set `IB_API_ARCHIVE` to a local/remote zip or manually unpack to `native/third_party/tws-api/`; or use the GitHub repo with `-DTWS_API_SOURCE_DIR=/path/to/tws-api`)

All artifacts land in `native/third_party/cache/` and remain untracked.

### TWS API (Manual Installation Required)

The Interactive Brokers TWS C++ API can be used in two ways:

1. **GitHub repo (recommended):** Clone [InteractiveBrokers/tws-api](https://github.com/InteractiveBrokers/tws-api) next to this repo (e.g. `../tws-api`). CMake auto-detects it and builds the client with `-DTWS_API_BUILD_VENDOR=ON`.
2. **IBKR zip:** Download from https://interactivebrokers.github.io/, extract to `native/third_party/tws-api/` (zip layout: `IBJts/source/cppclient/client/`).

**Note**: The current implementation uses stub functions. Full TWS API integration requires implementing the actual IBKR client callbacks.

## Installation

### 1. Install Build Tools

```bash
# macOS (using Homebrew)
brew install cmake

# Verify installation
cmake --version  # Should be 3.21 or higher
```

### 2. Clone Repository

```bash
git clone <repository-url>
cd ib-box-spread-generator
```

### 3. Download TWS API (Optional but Recommended)

```bash
# Create third-party directory
mkdir -p native/third_party/tws-api

# Download TWS API from IBKR and extract to native/third_party/tws-api/
# Directory structure should be:
# native/third_party/tws-api/source/cppclient/client/*.h
```

### 4. Build the Project

```bash
# Standard build
chmod +x scripts/build_universal.sh
./scripts/build_universal.sh

# Or use fast build with ccache (10-100x faster rebuilds)
brew install ccache  # Install ccache first
./scripts/build_fast.sh

# Or distributed build (2-10x faster clean builds)
export DISTCC_HOSTS="localhost/4 remote-ip/8"
./scripts/build_distributed.sh
```

From repo root you can also use **Make** (wraps CMake presets): `make build`, `make test`, `make lint` (see `make help`). Or **CMake** directly: `cmake --build build --target lint` from a configured build dir.

The binary will be created at: `build/bin/ib_box_spread`

**Build Optimization** (see `docs/DISTRIBUTED_COMPILATION.md` for details):

- 🚀 **ccache**: Cache compilation results (10-100x speedup on rebuilds)
- 🌐 **distcc**: Distribute compilation across network (2-10x speedup)
- ⚡ **Both**: Use together for maximum performance

### 5. Configure

```bash
# Copy example configuration
cp config/config.example.json config/config.json
# Edit with your settings
nano config/config.json  # or vim, code, etc.
```

When installed via Homebrew, the CLI searches for a user copy of `config.json` in the following
locations (in priority order) before falling back to the `--config` flag:

- `$HOME/.config/ib_box_spread/config.json`
- `$HOME/Library/Application Support/ib_box_spread/config.json` (macOS)
- `/usr/local/etc/ib_box_spread/config.json`
- `/etc/ib_box_spread/config.json`

Copy the packaged example into one of those directories to get started:

```bash
mkdir -p "${HOME}/.config/ib_box_spread"
cp "$(brew --prefix)/share/ib-box-spread/config.example.json" \
   "${HOME}/.config/ib_box_spread/config.json"

# Or have the CLI generate a starter config for you
ib_box_spread --init-config
# Optional: specify a path explicitly
ib_box_spread --init-config /tmp/my_ib_box_spread.json
```

You can also set `IB_BOX_SPREAD_CONFIG=/path/to/config.json` to point both the C++ CLI and Python
orchestration script at an alternate location.

## Configuration

Edit `config/config.json`:

```json
{
  "tws": {
    "host": "127.0.0.1",
    "port": 7497,
    "client_id": 1
  },
  "strategy": {
    "symbols": ["SPY", "QQQ", "IWM"],
    "min_arbitrage_profit": 0.1,
    "min_roi_percent": 0.5
  },
  "risk": {
    "max_total_exposure": 50000.0,
    "max_positions": 10,
    "max_daily_loss": 2000.0
  },
  "dry_run": true,
  "logging": {
    "log_file": "logs/ib_box_spread.log",
    "log_level": "info",
    "log_to_console": true,
    "use_colors": true
  },
  "connection_management": {
    "weekly_reauth": {
      "enabled": false,
      "day_of_week": "sunday",
      "time_utc": "21:00",
      "auto_reconnect": true
    }
  },
  "notifications": {
    "enabled": false,
    "channels": [
      {
        "type": "email",
        "smtp_host": "smtp.example.com",
        "from": "alerts@example.com",
        "to": ["ops@example.com"],
        "events": ["reauth_failure", "order_rejected"]
      }
    ]
  },
  "data_providers": {
    "primary": "ib",
    "fallbacks": ["orats"]
  },
  "orats": {
    "enabled": false,
    "api_token": ""
  },
  "ibkr_portal": {
    "enabled": false,
    "base_url": "https://localhost:5001/v1/portal",
    "verify_ssl": false,
    "preferred_accounts": []
  }
}
```

### Connection Management

- `weekly_reauth.enabled`: Enable the weekly IB Key re-auth workflow.
- `weekly_reauth.time_utc`: UTC time to start the re-authentication window.
- `weekly_reauth.auto_reconnect`: Automatically reconnect after 2FA succeeds (otherwise waits for manual reconnect).

When enabled, the runtime pauses the strategy, disconnects IB, prompts for re-auth, and resumes once the session is refreshed.

### Notifications

Channels subscribe to specific events (e.g., `reauth_failure`, `order_rejected`, `strategy_paused`). Supported types:

- **email** (SMTP credentials + recipients)
- **webhook** (generic HTTPS POST)
- **sms** (Twilio account credentials)
- **telegram** (bot token + chat id)

### Market Data Providers

Configure a primary provider and optional fallbacks. The router will attempt each provider in sequence until a fresh quote is available. Setting `fallbacks` to `orats` enables ORATS core data as a backup when IB quotes are stale or unavailable.

### QuestDB Archiving

- `questdb.enabled`: Persist validated quotes and trades to QuestDB via the ILP wire protocol.
- `questdb.quote_table` / `questdb.trade_table`: Destination table names.
- Ensure QuestDB is running with ILP enabled (default port `9009`). The runtime retries transient failures and logs deliverability issues.

### IBKR Client Portal API

- `ibkr_portal.enabled`: Toggle the REST integration (requires the Client Portal Gateway running locally).
- `ibkr_portal.base_url`: Base URL to the Gateway (defaults to `https://localhost:5001/v1/portal`).
- `ibkr_portal.verify_ssl`: Set to `false` when using the self-signed certificate shipped with the Gateway.
- `ibkr_portal.preferred_accounts`: Optional list of account IDs to prioritise when multiple accounts are returned.

When enabled, the strategy validates the session during startup and logs account net liquidation/buying power via the Client Portal summary endpoint. Start IBKR’s Client Portal Gateway (for example, by running `bin/run.sh` from the Gateway package) before launching the strategy.

### Configuration Parameters

#### TWS Connection

- `host`: TWS/Gateway hostname (usually "127.0.0.1")
- `port`:
  - **7497** = Paper Trading (Safe for testing)
  - **7496** = Live Trading (Real money!)
- `client_id`: Unique client identifier (1-32)

#### Strategy

- `symbols`: List of underlying symbols to monitor
- `min_arbitrage_profit`: Minimum profit in dollars
- `min_roi_percent`: Minimum return on investment (%)
- `max_position_size`: Maximum capital per trade ($)
- `min_days_to_expiry`: Minimum days to expiration
- `max_days_to_expiry`: Maximum days to expiration

#### Risk Management

- `max_total_exposure`: Maximum total capital deployed
- `max_positions`: Maximum number of open positions
- `max_daily_loss`: Stop trading if daily loss exceeds this

## Usage

### Paper Trading (Recommended First)

```bash
# Make sure config.json has:
# - "port": 7497 (paper trading)
# - "dry_run": true

# Start TWS or IB Gateway in Paper Trading mode

# Run the application
./build/bin/ib_box_spread --config config/config.json --dry-run
```

### Live Trading (Use with Extreme Caution)

```bash
# Update config.json:
# - "port": 7496 (live trading)
# - "dry_run": false

# Start TWS or IB Gateway in Live mode

# Run with live trading (BE VERY CAREFUL!)
./build/bin/ib_box_spread --config config/config.json

# You can still override with --dry-run flag for safety
./build/bin/ib_box_spread --config config/config.json --dry-run
```

### Command-Line Options

```bash
# Show help
./build/bin/ib_box_spread --help

# Validate configuration without trading
./build/bin/ib_box_spread --config config/config.json --validate

# Override log level
./build/bin/ib_box_spread --log-level debug

# Use custom config file
./build/bin/ib_box_spread --config /path/to/custom/config.json
```

## Testing

### Run All Tests

**Python Tests:**
```bash
# Run all Python tests
./scripts/run_python_tests.sh

# Run with coverage
./scripts/run_python_tests.sh --coverage

# Run with HTML coverage report
./scripts/run_python_tests.sh --html
```

**C++ Tests:**
```bash
# Run test suite
cd native/build
ctest --output-on-failure

# Or use the build script
./scripts/build_universal.sh test
```

### Run Specific Test Categories

**Python:**
```bash
# Run specific test file
pytest python/tests/test_security.py

# Run with coverage for specific module
pytest python/tests/test_security.py --cov=python/services/security --cov-report=term
```

**C++:**
```bash
cd native/build
./box_spread_tests "[config]"  # Only config tests
./box_spread_tests "[strategy]"  # Only strategy tests
./box_spread_tests "[risk]"  # Only risk tests
```

### Test Coverage

**Coverage Target**: 30%+ overall coverage

**Generate Coverage Reports:**
```bash
# Generate Python coverage
./scripts/generate_python_coverage.sh --html

# Generate C++ coverage (when libraries available)
./scripts/generate_cpp_coverage.sh

# Generate combined coverage
./scripts/generate_coverage.sh --html
```

**Coverage Reports:**
- **Python HTML**: `htmlcov/index.html`
- **C++ HTML**: `native/build-coverage/coverage_html/index.html`

**Documentation:**
- See [Test Coverage Setup Guide](docs/TEST_COVERAGE_SETUP.md) for detailed coverage instructions
- See [Coverage Gap Analysis](docs/COVERAGE_GAP_ANALYSIS.md) for prioritized test additions

The test suite includes:

- Configuration validation tests
- Box spread strategy tests
- Risk calculator tests
- Order manager tests
- Security and path validation tests
- Integration tests
- Edge case handling

## Extracted Components

This project has been split into multiple focused repositories for better organization and reuse:

### 🔧 Reusable Tools & Libraries

- **[box-spread-cpp](https://github.com/davidl71/box-spread-cpp)** - Broker-agnostic C++ library for box spread arbitrage calculations and risk management
- **[box-spread-python](https://github.com/davidl71/box-spread-python)** - Broker-agnostic Python utilities for box spread strategies (DSL, tools, TUI, ML)
- **[trading-api-docs](https://github.com/davidl71/trading-api-docs)** - Trading API documentation and integration guides
- **[trading-architecture-docs](https://github.com/davidl71/trading-architecture-docs)** - Trading system architecture and design documentation
- **[trading-setup-docs](https://github.com/davidl71/trading-setup-docs)** - Trading system setup, configuration, and deployment documentation
- **[trading-automation-docs](https://github.com/davidl71/trading-automation-docs)** - Trading project automation and maintenance documentation
- **[trading-tools-docs](https://github.com/davidl71/trading-tools-docs)** - Trading tools, frameworks, and best practices documentation
- **[trading-mcp-servers](https://github.com/davidl71/trading-mcp-servers)** - MCP servers for trading operations (broker-agnostic, extracted from the deprecated former monorepo path `mcp/trading_server/`)
- **[trading-build-tools](https://github.com/davidl71/trading-build-tools)** - Reusable CMake build scripts and presets for C++ trading projects
- **[trading-automation-tools](https://github.com/davidl71/trading-automation-tools)** - Project housekeeping and analysis automation tools
- **[box-spread-notebooks](https://github.com/davidl71/box-spread-notebooks)** - Educational Jupyter notebooks for box spread trading strategies

**Note**: These repositories are currently private. See [Project Split Strategy](docs/PROJECT_SPLIT_STRATEGY.md) for details on the modularization approach.

## Platform Documentation

### Core Platform

- **[Platform Overview](docs/platform/README.md)** - Synthetic Financing Platform architecture
- **[Investment Strategy Framework](docs/platform/INVESTMENT_STRATEGY_FRAMEWORK.md)** - Portfolio allocation framework
- **[Primary Goals and Requirements](docs/platform/PRIMARY_GOALS_AND_REQUIREMENTS.md)** - System objectives
- **[Synthetic Financing Architecture](docs/platform/SYNTHETIC_FINANCING_ARCHITECTURE.md)** - Multi-asset relationship system

### Strategy Modules

- **[Box Spread Strategy](docs/strategies/box-spread/README.md)** - Box spread strategy overview and documentation

## Project Structure

```
ib-box-spread-generator/
├── CMakeLists.txt              # Main CMake configuration
├── README.md                   # This file
├── .gitignore                  # Git ignore rules
│
├── config/
│   └── config.example.json     # Example configuration
│
├── include/                    # Header files
│   ├── types.h                # Common types and enums
│   ├── config_manager.h       # Configuration management
│   ├── tws_client.h           # TWS API wrapper
│   ├── option_chain.h         # Option chain structures
│   ├── box_spread_strategy.h  # Strategy implementation
│   ├── order_manager.h        # Order execution
│   └── risk_calculator.h      # Risk management
│
├── src/                        # Source files
│   ├── ib_box_spread.cpp      # Main entry point
│   ├── config_manager.cpp
│   ├── tws_client.cpp
│   ├── option_chain.cpp
│   ├── box_spread_strategy.cpp
│   ├── order_manager.cpp
│   └── risk_calculator.cpp
│
├── tests/                      # Test files
│   ├── CMakeLists.txt
│   ├── test_main.cpp
│   ├── test_config_manager.cpp
│   ├── test_box_spread_strategy.cpp
│   ├── test_risk_calculator.cpp
│   └── test_order_manager.cpp
│
├── scripts/
│   └── build_universal.sh      # Build script
│
├── docs/                       # Documentation
│   └── (additional docs)
│
└── logs/                       # Log files (created at runtime)
```

## Architecture

```
┌─────────────┐
│   User CLI  │
└──────┬──────┘
       │
┌──────▼────────────┐
│ Box Spread        │
│ Strategy Engine   │
└────┬─────┬────────┘
     │     │
     │     │    ┌──────────────┐
     │     └────► Risk         │
     │          │ Calculator   │
     │          └──────────────┘
     │
┌────▼──────────┐
│ Order Manager │
└───────┬───────┘
        │
┌───────▼───────┐
│  TWS Client   │
└───────┬───────┘
        │
┌───────▼───────────┐
│ IB Gateway / TWS  │
└───────────────────┘
        │
┌───────▼──────────┐
│    Exchange      │
└──────────────────┘
```

### Component Responsibilities

- **Box Spread Strategy**: Identifies arbitrage opportunities
- **Risk Calculator**: Manages position sizing and risk limits
- **Order Manager**: Handles order execution and tracking
- **TWS Client**: Interfaces with Interactive Brokers API
- **Config Manager**: Loads and validates configuration

## Development

### Building for Development

```bash
# Debug build with sanitizers
cmake -B build -DCMAKE_BUILD_TYPE=Debug -DENABLE_ASAN=ON
cmake --build build

# Run with debug logging
./build/bin/ib_box_spread --log-level debug
```

### Code Style

This project follows:

- Modern C++20 standards
- RAII resource management
- Error handling via exceptions
- Const-correctness
- Comprehensive logging

### Adding New Features

1. Add header declarations in `include/`
2. Implement in `src/`
3. Add tests in `tests/`
4. Update documentation
5. Test thoroughly in paper trading mode

## Security

### Important Security Practices

1. **Never commit credentials**:

   ```bash
   # config.json is in .gitignore
   # Never use `git add -f` on it!
   ```

2. **Use environment variables for sensitive data** (if needed):

   ```bash
   export IBKR_USERNAME="your_username"
   export IBKR_PASSWORD="your_password"
   ```

3. **Review .gitignore**:
   - config/config.json ✓
   - \*.env ✓
   - logs/ ✓
   - credentials/ ✓

4. **Start in dry-run mode**: Always test with `"dry_run": true`

5. **Use paper trading first**: Port 7497, never 7496 initially

## Troubleshooting

### Build Issues

**Problem**: C++ standard library headers not found (`'string' file not found`, `'mutex' file not found`)

Common on macOS when Xcode Command Line Tools are missing or incomplete. The build injects the SDK path when possible; if it still fails:

```bash
# Fix: Install or repair Command Line Tools
xcode-select --install
# Then verify
./scripts/verify_toolchain.sh
```

See **[docs/BUILD_FAILURES_AND_DEPENDENCIES.md](docs/BUILD_FAILURES_AND_DEPENDENCIES.md)** for full details (Protobuf alignment, other deps).

**Problem**: CMake version too old

```bash
# Solution: Update CMake
brew upgrade cmake
```

**Problem**: Compiler doesn't support C++20

```bash
# Solution: Update compiler
brew install llvm  # or gcc
```

**Problem**: TWS API not found

```bash
# Solution: Download TWS API and place in native/third_party/tws-api/
# Or disable TWS API in CMakeLists.txt for testing
```

### Runtime Issues

**Problem**: "Failed to connect to TWS"

```
Solutions:
1. Ensure TWS or IB Gateway is running
2. Check port number (7497 vs 7496)
3. Enable API connections in TWS settings:
   - Configure → API → Settings
   - Enable ActiveX and Socket Clients
   - Add 127.0.0.1 to trusted IPs
```

**Problem**: "Configuration validation failed"

```
Solution: Check config.json for:
- Valid JSON syntax
- All required fields present
- Reasonable values for limits
```

**Problem**: No opportunities found

```
Solutions:
1. Lower min_arbitrage_profit threshold
2. Expand symbols list
3. Adjust min/max days to expiry range
4. Check market hours (options only trade during market hours)
```

## Performance

### Expected Performance

- **CPU Usage**: Low (< 5% on modern CPU)
- **Memory**: ~50-100 MB
- **Network**: Minimal (periodic API calls)
- **Latency**: Depends on TWS connection

### Optimization Tips

1. **Reduce symbol list**: Monitor fewer symbols
2. **Increase loop delay**: Less frequent checks
3. **Filter by volume**: Only liquid options
4. **Adjust DTE range**: Narrower expiry range

## Roadmap

### Completed ✓

- [x] Project structure and build system
- [x] Configuration management
- [x] Core strategy framework
- [x] Risk management system
- [x] Order management framework
- [x] Test infrastructure
- [x] Documentation

### In Progress 🚧

- [ ] Full TWS API integration
- [ ] Black-Scholes Greeks calculation
- [ ] Historical data analysis

### Planned 📋

- [ ] Web dashboard for monitoring
- [ ] Database for trade history
- [ ] Performance analytics
- [ ] Multi-strategy support
- [ ] Alert/notification system
- [ ] Machine learning for opportunity scoring

- [x] Cython bindings for C++ calculations

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests
5. Ensure tests pass (`ctest`)
6. Commit (`git commit -m 'Add amazing feature'`)
7. Push (`git push origin feature/amazing-feature`)
8. Open a Pull Request

### Code Review Checklist

- [ ] Code follows C++20 standards
- [ ] Tests added and passing
- [ ] Documentation updated
- [ ] No credentials in code
- [ ] Logging added for important operations
- [ ] Error handling implemented

## License

MIT License - See [LICENSE](LICENSE) file

## Disclaimer

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

**Trading involves substantial risk of loss**. Past performance is not indicative of future results. Only risk capital that you can afford to lose should be used for trading.

## Support

- **Issues**: https://github.com/yourusername/ib-box-spread-generator/issues
- **Discussions**: https://github.com/yourusername/ib-box-spread-generator/discussions
- **Email**: your.email@example.com

## Acknowledgments

- Interactive Brokers for their API
- The C++ community for excellent libraries
- Options trading community for strategy insights

## Resources

### Options Trading

- [IBKR Options Trading](https://www.interactivebrokers.com/en/trading/options.php)
- [Box Spread Strategy](https://www.investopedia.com/terms/b/boxspread.asp)
- [Options Greeks](https://www.investopedia.com/trading/getting-to-know-the-greeks/)

### Technical Documentation

- [CMake Documentation](https://cmake.org/documentation/)
- [C++20 Features](https://en.cppreference.com/w/cpp/20)
- [Catch2 Tutorial](https://github.com/catchorg/Catch2/blob/devel/docs/tutorial.md)

---

**Remember**: Always test in paper trading mode first! Never risk money you can't afford to lose.
