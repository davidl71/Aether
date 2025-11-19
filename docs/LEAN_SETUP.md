# LEAN Setup Guide

**Date**: 2025-11-18
**Status**: In Progress
**Purpose**: Guide for setting up LEAN (QuantConnect LEAN) for box spread trading

---

## Overview

This guide covers the installation and configuration of LEAN for use with the IB Box Spread Generator. LEAN provides multi-broker support (IBKR, Alpaca, etc.) and integrates with our existing C++ calculations via Cython.

---

## Prerequisites

### Required

- **Python 3.11+** (✅ Verified: Python 3.14.0 available)
- **pip** (Python package manager)
- **Cython 3.0+** (for C++ bindings - already installed)

### Optional (for C# algorithms)

- **.NET SDK 6.0+** (only needed if using C# algorithms, not required for Python)
- **Mono** (alternative .NET runtime for macOS/Linux)

---

## Installation Steps

### 1. Install LEAN CLI

```bash
# Install LEAN CLI globally
pip3 install lean --user

# Or install in virtual environment
python3 -m venv venv
source venv/bin/activate  # On macOS/Linux
pip install lean
```

**Verify installation:**
```bash
lean --version
```

### 2. Install .NET SDK (Optional - for C# support)

**macOS:**
```bash
# Using Homebrew
brew install --cask dotnet

# Or download from: https://dotnet.microsoft.com/download
```

**Verify .NET installation:**
```bash
dotnet --version
```

**Note**: Python algorithms don't require .NET SDK, but LEAN's core engine uses C#. For Python-only development, you may not need .NET installed.

### 3. Initialize LEAN Project

```bash
# Create LEAN project directory
mkdir -p lean_project
cd lean_project

# Initialize LEAN project
lean init

# This creates:
# - Main/ (algorithm files)
# - config.json (configuration)
# - data/ (data storage)
```

### 4. Configure LEAN

```bash
# Configure LEAN (interactive)
lean config

# Or edit config.json directly
```

**Configuration options:**
- Data provider (QuantConnect, local files, etc.)
- Broker settings (IBKR, Alpaca, etc.)
- Logging and debugging options

---

## Project Structure

```
python/
├── lean/                    # LEAN integration package
│   ├── __init__.py
│   ├── box_spread_algorithm.py    # Main LEAN algorithm
│   ├── data_converter.py           # LEAN ↔ C++ data conversion
│   ├── type_mappings.py            # Type mapping utilities
│   └── strategy_config.py          # Strategy configuration
├── bindings/                # Cython bindings (existing)
│   └── box_spread_bindings.pyx
└── integration/             # NautilusTrader (to be deprecated)
```

---

## Integration Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    C++ Core (Existing)                      │
│  - Box spread calculations                                  │
│  - Risk calculator                                          │
│  - Option chain scanner                                     │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       │ Cython (existing)
                       │
┌──────────────────────▼──────────────────────────────────────┐
│                  Python Strategy Layer                      │
│  - LEAN Algorithm class                                     │
│  - Data conversion (LEAN ↔ C++)                            │
│  - Strategy logic                                           │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       │ LEAN Python API
                       │
┌──────────────────────▼──────────────────────────────────────┐
│                    LEAN Engine (C#)                         │
│  - Multi-broker support (IBKR, Alpaca)                     │
│  - Market data handling                                     │
│  - Order execution                                          │
│  - Position management                                      │
└─────────────────────────────────────────────────────────────┘
```

---

## Testing LEAN Installation

### 1. Run Sample Algorithm

```bash
# Create a simple test algorithm
cat > Main/main.py << 'EOF'
from AlgorithmImports import *

class TestAlgorithm(QCAlgorithm):
    def Initialize(self):
        self.SetStartDate(2025, 1, 1)
        self.SetCash(100000)
        self.AddEquity("SPY", Resolution.Minute)

    def OnData(self, data):
        if not self.Portfolio.Invested:
            self.SetHoldings("SPY", 1.0)
EOF

# Run backtest
lean backtest "Main/main.py"
```

### 2. Verify Broker Connections

```bash
# Test IBKR connection (paper trading)
lean live "Main/main.py" --brokerage InteractiveBrokers --data-provider InteractiveBrokers

# Test Alpaca connection
lean live "Main/main.py" --brokerage Alpaca --data-provider Alpaca
```

---

## Configuration Files

### LEAN config.json

```json
{
  "algorithm-type-name": "BoxSpreadAlgorithm",
  "algorithm-language": "Python",
  "algorithm-location": "Main/box_spread_algorithm.py",
  "data-folder": "data/",
  "results-folder": "results/",
  "job-queue-handler": "QuantConnect.Queues.JobQueue",
  "api-handler": "QuantConnect.Api.Api",
  "map-file-provider": "QuantConnect.Data.Auxiliary.LocalDiskMapFileProvider",
  "factor-file-provider": "QuantConnect.Data.Auxiliary.LocalDiskFactorFileProvider",
  "data-provider": "QuantConnect.Lean.Engine.DataFeeds.DefaultDataProvider",
  "alpha-handler": "QuantConnect.Lean.Engine.Alphas.DefaultAlphaHandler",
  "data-channel-provider": "DataChannelProvider",
  "object-store": "QuantConnect.Lean.Engine.Storage.LocalObjectStore",
  "data-aggregator": "QuantConnect.Lean.Engine.DataFeeds.AggregationManager",
  "symbol-minute-limit": 10000,
  "symbol-second-limit": 10000,
  "job-queue": {
    "job-queue-type": "QuantConnect.Queues.JobQueue",
    "maximum-queue-size": 20
  },
  "api": {
    "api-handler": "QuantConnect.Api.Api",
    "job-queue-handler": "QuantConnect.Queues.JobQueue",
    "data-queue-handler": "QuantConnect.Queues.JobQueue",
    "data-provider": "QuantConnect.Lean.Engine.DataFeeds.DefaultDataProvider",
    "file-provider": "QuantConnect.Lean.Engine.DataFeeds.DefaultDataProvider",
    "map-file-provider": "QuantConnect.Data.Auxiliary.LocalDiskMapFileProvider",
    "factor-file-provider": "QuantConnect.Data.Auxiliary.LocalDiskFactorFileProvider"
  },
  "algorithm-handler": {
    "algorithm-type-name": "BoxSpreadAlgorithm",
    "algorithm-language": "Python",
    "algorithm-location": "Main/box_spread_algorithm.py"
  },
  "data-folder": "data/",
  "results-folder": "results/",
  "cache-folder": "cache/",
  "version-id": "",
  "algorithm-id": "",
  "algorithm-version": "",
  "algorithm-name": "Box Spread Strategy",
  "algorithm-description": "Box spread arbitrage strategy using LEAN",
  "algorithm-tags": "box-spread,arbitrage,options"
}
```

### Broker Configuration

**IBKR Configuration:**
```json
{
  "brokerage": {
    "type": "InteractiveBrokers",
    "host": "127.0.0.1",
    "port": 7497,
    "account": "DU123456",
    "username": "",
    "password": "",
    "trading-mode": "paper"
  }
}
```

**Alpaca Configuration:**
```json
{
  "brokerage": {
    "type": "Alpaca",
    "key-id": "YOUR_API_KEY",
    "secret-key": "YOUR_SECRET_KEY",
    "base-url": "https://paper-api.alpaca.markets",
    "trading-mode": "paper"
  }
}
```

---

## Next Steps

1. ✅ **Install LEAN CLI** (this guide)
2. ⏳ **Research LEAN broker adapters** (T-40)
3. ⏳ **Design LEAN strategy architecture** (T-41)
4. ⏳ **Implement data conversion layer** (T-42)
5. ⏳ **Implement LEAN box spread strategy** (T-43)
6. ⏳ **Configure IBKR integration** (T-45)
7. ⏳ **Configure Alpaca integration** (T-46)

---

## Troubleshooting

### LEAN CLI Not Found

```bash
# Add user bin to PATH (macOS/Linux)
export PATH="$HOME/.local/bin:$PATH"

# Or reinstall with --user flag
pip3 install --user lean
```

### .NET SDK Issues

**If you only use Python algorithms:**
- .NET SDK is optional
- LEAN's Python API works without .NET
- Only needed for C# algorithm development

**If you need .NET:**
```bash
# macOS
brew install --cask dotnet

# Verify
dotnet --version
```

### Broker Connection Issues

- **IBKR**: Ensure TWS/IB Gateway is running and API is enabled
- **Alpaca**: Verify API keys and base URL (paper vs live)
- Check LEAN logs in `results/` directory

---

## References

- [LEAN GitHub](https://github.com/QuantConnect/Lean)
- [LEAN CLI Documentation](https://www.quantconnect.com/docs/v2/lean-cli)
- [LEAN IBKR Integration](https://www.quantconnect.com/docs/v2/lean-cli/live-trading/brokerages/interactive-brokers)
- [LEAN Alpaca Integration](https://www.quantconnect.com/docs/v2/lean-cli/live-trading/brokerages/alpaca)

---

## Known Issues

### Python 3.14 Compatibility

**Issue**: LEAN CLI 1.0.221 has a compatibility issue with Python 3.14 due to Pydantic v1:
```
Core Pydantic V1 functionality isn't compatible with Python 3.14 or greater.
```

**Workaround**: Use Python 3.11 or 3.12 for LEAN development:

```bash
# Install Python 3.12 via Homebrew
brew install python@3.12

# Create virtual environment with Python 3.12
python3.12 -m venv python/venv312
source python/venv312/bin/activate
pip install lean
```

**Status**: Waiting for LEAN to update Pydantic dependency or provide Python 3.14 support.

---

## Status

- ✅ Python 3.12 available and working
- ✅ LEAN CLI installed in Python 3.12 virtual environment (`python/venv312/`)
- ✅ LEAN CLI verified working (`lean --version`)
- ⏳ .NET SDK check (optional - not required for Python algorithms)
- ⏳ LEAN project initialization
- ⏳ Configuration setup

**Virtual Environment:**
- Python 3.12: `python/venv312/` (for LEAN)
- Python 3.14: `python/venv/` (for other development)
