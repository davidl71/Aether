# Quick Start Guide

Get up and running with IBKR Box Spread Generator in minutes.

## ⚠️ Safety First

**READ THIS BEFORE STARTING:**

- This is **trading software** - real money is at risk
- **ALWAYS** start with paper trading (port 7497)
- **ALWAYS** enable dry-run mode initially
- Test for **30+ days** before considering live trading
- Start with **small positions** ($500 max) when going live

## Installation (Choose One Method)

### Method 1: Homebrew (Easiest) ⭐ Recommended

```bash
# Add tap (private repo - requires SSH access)
brew tap davidl71/ib-box-spread git@github.com:davidl71/homebrew-ib-box-spread.git

# Install main binary
brew install davidl71/ib-box-spread/ib-box-spread

# Install TUI (Terminal User Interface)
brew install davidl71/ib-box-spread/ib-box-spread-tui

# Verify installation
ib_box_spread --help
ib-box-spread-tui --help
```

### Method 2: From Source

```bash
# Clone repository
git clone git@github.com:davidl71/ib_box_spread_full_universal.git
cd ib_box_spread_full_universal

# Build (requires CMake, Ninja, and dependencies)
./scripts/build_universal.sh

# Or use fast build with ccache
./scripts/build_fast.sh

# Binary will be at: build/bin/ib_box_spread
```

### Method 3: Using Worktree (For Development)

```bash
# Set up worktree with dependencies pre-built
./scripts/setup_worktree.sh my-worktree

# This will:
# - Create worktree
# - Build Intel Decimal library
# - Build TWS API library
# - Configure main project
```

## Configuration (2 Minutes)

### Step 1: Create Config File

```bash
# Copy example config
cp config/config.example.json config/config.json

# Edit with your settings
nano config/config.json  # or use your favorite editor
```

### Step 2: Essential Settings

**Minimum configuration** (safe defaults):

```json
{
  "tws": {
    "host": "127.0.0.1",
    "port": 7497,        // 7497 = Paper Trading (SAFE)
    "client_id": 1
  },
  "strategy": {
    "symbols": ["SPY"],
    "min_arbitrage_profit": 0.10,
    "min_roi_percent": 0.5,
    "max_position_size": 1000.0
  },
  "risk": {
    "max_total_exposure": 5000.0,
    "max_positions": 3,
    "max_daily_loss": 500.0
  },
  "dry_run": true       // CRITICAL: Keep this true for testing!
}
```

**Important Settings:**

- `port: 7497` = Paper Trading (safe)
- `port: 7496` = Live Trading (real money!)
- `dry_run: true` = Simulate without executing trades

## First Run (5 Minutes)

### Step 1: Validate Configuration

```bash
# Check config is valid
ib_box_spread --config config/config.json --validate
```

### Step 2: Run in Dry-Run Mode

```bash
# Safe test run (no real trades)
ib_box_spread --config config/config.json --dry-run

# Or use TUI for visual monitoring
ib-box-spread-tui --mock  # Test with mock data
```

### Step 3: Check Logs

```bash
# View logs
tail -f logs/ib_box_spread.log

# Or check recent entries
tail -n 50 logs/ib_box_spread.log
```

## Using the TUI

### Start TUI

```bash
# With mock data (offline testing)
ib-box-spread-tui --mock

# Connect to backend (requires running backend)
ib-box-spread-tui

# Custom endpoint
ib-box-spread-tui --endpoint http://localhost:8080/api/snapshot
```

### TUI Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Tab` | Switch tabs |
| `S` | Start strategy |
| `T` | Stop strategy |
| `D` | Toggle dry-run |
| `?` | Show help |
| `Q` | Quit |

**Full shortcuts**: Press `?` in TUI or see `tui/docs/KEYBOARD_SHORTCUTS.md`

## Testing

### Run Test Suite

```bash
# From build directory
cd build
ctest --output-on-failure

# Or from project root
./scripts/test_tui.sh --short  # Fast tests
```

### Expected Output

```
✅ All tests should pass
✅ Configuration validation works
✅ Dry-run mode works
✅ Logging works
```

## Common First Tasks

### 1. Test Configuration

```bash
ib_box_spread --config config/config.json --validate
```

### 2. Run Dry-Run Test

```bash
ib_box_spread --config config/config.json --dry-run
```

### 3. Monitor with TUI

```bash
# Terminal 1: Start backend (if you have one)
# Terminal 2: Start TUI
ib-box-spread-tui --mock
```

### 4. Check System Status

Look for these indicators in TUI header:

- `TWS: OK` - TWS connection active
- `Mode: DRY-RUN` - Safe mode enabled
- `Strategy: RUNNING` - Strategy active

## Next Steps

### For Development

1. **Explore Codebase**:

   ```bash
   # Main entry point
   native/src/ib_box_spread.cpp

   # Strategy logic
   native/src/box_spread_strategy.cpp

   # Configuration
   native/src/config_manager.cpp
   ```

2. **Read Documentation**:
   - `docs/CODEBASE_ARCHITECTURE.md` - System design
   - `docs/COMMON_PATTERNS.md` - Coding patterns
   - `docs/API_DOCUMENTATION_INDEX.md` - API reference

3. **Run Tests**:

   ```bash
   cd build && ctest --output-on-failure
   ```

### For Trading

1. **Paper Trading Setup**:
   - Install TWS or IB Gateway
   - Enable API access (port 7497)
   - Test connection

2. **Configuration**:
   - Start with conservative settings
   - Use small position sizes
   - Enable all safety features

3. **Validation Period**:
   - Run for 30+ days in paper trading
   - Monitor all trades
   - Review performance metrics
   - Only then consider live trading

## Troubleshooting

### Binary Not Found

```bash
# Check installation
which ib_box_spread

# Reinstall
brew reinstall ib-box-spread
```

### Configuration Errors

```bash
# Validate config
ib_box_spread --config config/config.json --validate

# Check JSON syntax
cat config/config.json | python -m json.tool
```

### Connection Issues

```bash
# Check TWS is running
# Check port (7497 = paper, 7496 = live)
# Check API is enabled in TWS settings
```

### Build Failures

```bash
# Check dependencies
brew install cmake ninja protobuf abseil

# Clean and rebuild
rm -rf build
./scripts/build_universal.sh
```

## Shell Completions (Optional but Recommended)

Enable tab-completion for faster CLI usage:

```bash
# Generate and install completions
./scripts/generate_completions.sh
./scripts/install_completions.sh

# Or install for your shell only
./scripts/install_completions.sh auto  # Auto-detects shell
```

After installation, use `<TAB>` to complete options:

```bash
ib_box_spread --<TAB>  # Shows all available options
ib_box_spread --log-level <TAB>  # Shows: trace debug info warn error
```

See `docs/SHELL_COMPLETION.md` for detailed instructions.

## Quick Reference

### Essential Commands

```bash
# Build
./scripts/build_universal.sh

# Test
cd build && ctest --output-on-failure

# Run (dry-run)
ib_box_spread --config config/config.json --dry-run

# Validate config
ib_box_spread --config config/config.json --validate

# TUI
ib-box-spread-tui --mock
```

### Important Files

| File | Purpose |
|------|---------|
| `config/config.json` | Your configuration |
| `config/config.example.json` | Example configuration |
| `logs/ib_box_spread.log` | Application logs |
| `build/bin/ib_box_spread` | Main binary (if built from source) |

### Important Ports

| Port | Purpose |
|------|---------|
| `7497` | Paper Trading (safe) |
| `7496` | Live Trading (real money!) |

## Safety Checklist

Before running:

- [ ] `dry_run: true` in config
- [ ] `port: 7497` (paper trading)
- [ ] Small position sizes configured
- [ ] Risk limits set appropriately
- [ ] Logging enabled
- [ ] TWS/Gateway running (if connecting)

## Getting Help

- **Documentation**: See `docs/` directory
- **Man Pages**: `man ib-box-spread-tui`
- **TUI Help**: Press `?` in TUI
- **Issues**: Check logs in `logs/`

## What's Next?

1. ✅ **Install** - You're here!
2. ⏳ **Configure** - Set up `config/config.json`
3. ⏳ **Test** - Run in dry-run mode
4. ⏳ **Validate** - Test for 30+ days
5. ⏳ **Monitor** - Use TUI to watch activity
6. ⏳ **Paper Trade** - Connect to TWS paper trading
7. ⏳ **Live Trade** - Only after extensive testing

---

**Remember**: This is trading software. You can lose money. Always test thoroughly before live trading.

**Good luck!** 🚀
