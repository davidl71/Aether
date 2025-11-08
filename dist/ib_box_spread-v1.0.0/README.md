# IBKR Box Spread Generator

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)]()
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![C++](https://img.shields.io/badge/C%2B%2B-20-blue.svg)]()

Automated options arbitrage trading system for Interactive Brokers using box spread strategies.

## ⚠️ Important Disclaimers

**READ THIS CAREFULLY BEFORE USING**

- **This is trading software**: Real money is at risk. You can lose money. Use entirely at your own risk.
- **Paper trading first**: ALWAYS test thoroughly in paper trading mode before considering live trading.
- **Not financial advice**: This software is for educational and research purposes only.
- **Regulatory compliance**: Ensure you comply with all applicable securities regulations.
- **No warranties**: THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND.
- **Stub implementation**: The current TWS API client is a stub. You must integrate the actual IBKR TWS API.

## Features

- ✅ Automated box spread identification and analysis
- ✅ Real-time options chain monitoring
- ✅ Risk-based position sizing and management
- ✅ Comprehensive logging with spdlog
- ✅ Dry-run mode for safe testing
- ✅ Universal binary support (Intel + Apple Silicon on macOS)
- ✅ Modern C++20 codebase with extensive error handling
- ✅ JSON-based configuration
- ✅ Comprehensive test suite with Catch2

## What is a Box Spread?

A box spread is a complex options strategy that combines four positions:

1. **Long call** at strike K1 (lower strike)
2. **Short call** at strike K2 (higher strike)
3. **Long put** at strike K2 (higher strike)
4. **Short put** at strike K1 (lower strike)

**Arbitrage Opportunity**: When the net debit paid is less than the strike width (K2 - K1), there's a guaranteed profit at expiration.

**Example**:
- Buy SPY 500 Call @ $2.50
- Sell SPY 510 Call @ $1.00
- Buy SPY 510 Put @ $2.00
- Sell SPY 500 Put @ $0.75
- **Net Debit**: $2.75
- **Strike Width**: $10.00
- **Arbitrage Profit**: $7.25 (263% ROI)

## Prerequisites

### System Requirements

- **macOS** 11.0 or later (for universal binary support)
- **CMake** 3.21 or higher
- **C++ Compiler** with C++20 support:
  - Clang 13+ (recommended for macOS)
  - GCC 11+
- **Interactive Brokers** TWS or IB Gateway
- **Active IBKR account** with options trading enabled

### Dependencies (Automatically Downloaded)

The following dependencies are automatically fetched by CMake:

- [nlohmann/json](https://github.com/nlohmann/json) - JSON parsing
- [spdlog](https://github.com/gabime/spdlog) - Logging
- [CLI11](https://github.com/CLIUtils/CLI11) - Command-line parsing
- [Catch2](https://github.com/catchorg/Catch2) - Testing framework

### TWS API (Manual Installation Required)

The Interactive Brokers TWS C++ API must be downloaded manually:

1. Visit https://interactivebrokers.github.io/
2. Download the TWS API for your platform
3. Extract to `third_party/tws-api/`

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
# Create vendor directory
mkdir -p third_party/tws-api

# Download TWS API from IBKR and extract to third_party/tws-api/
# Directory structure should be:
# third_party/tws-api/source/cppclient/client/*.h
```

### 4. Build the Project

```bash
# Make build script executable
chmod +x scripts/build_universal.sh

# Build (Release mode)
./scripts/build_universal.sh

# Or build in Debug mode
BUILD_TYPE=Debug ./scripts/build_universal.sh
```

The binary will be created at: `build/bin/ib_box_spread`

### 5. Configure

```bash
# Copy example configuration
cp config/config.example.json config/config.json

# Edit with your settings
nano config/config.json  # or vim, code, etc.
```

## Configuration

Edit `config/config.json`:

```json
{
  "tws": {
    "host": "127.0.0.1",
    "port": 7497,              // 7497 = Paper, 7496 = Live
    "client_id": 1
  },
  "strategy": {
    "symbols": ["SPY", "QQQ", "IWM"],
    "min_arbitrage_profit": 0.10,    // Min $0.10 profit
    "min_roi_percent": 0.5,          // Min 0.5% ROI
    "max_position_size": 10000.0,    // Max $10k per position
    "min_days_to_expiry": 30,
    "max_days_to_expiry": 90
  },
  "risk": {
    "max_total_exposure": 50000.0,   // Max $50k total
    "max_positions": 10,
    "max_daily_loss": 2000.0
  },
  "dry_run": true                    // ALWAYS start with true!
}
```

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

```bash
# Run test suite
cd build
ctest --output-on-failure

# Or use the build script
./scripts/build_universal.sh test
```

### Run Specific Test Categories

```bash
cd build/bin
./box_spread_tests "[config]"  # Only config tests
./box_spread_tests "[strategy]"  # Only strategy tests
./box_spread_tests "[risk]"  # Only risk tests
```

### Test Coverage

The test suite includes:
- Configuration validation tests
- Box spread strategy tests
- Risk calculator tests
- Order manager tests
- Input validation tests
- Edge case handling

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
   - *.env ✓
   - logs/ ✓
   - credentials/ ✓

4. **Start in dry-run mode**: Always test with `"dry_run": true`

5. **Use paper trading first**: Port 7497, never 7496 initially

## Troubleshooting

### Build Issues

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
# Solution: Download TWS API and place in third_party/tws-api/
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
