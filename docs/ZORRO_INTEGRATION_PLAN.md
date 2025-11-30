# Zorro Trading Platform Integration Plan

**Date**: 2025-11-30
**Source**: <https://zorro-project.com/>
**Purpose**: Comprehensive integration plan for leveraging Zorro's backtesting, optimization, and visualization capabilities with the IBKR box spread arbitrage system

---

## Executive Summary

Zorro is a free, institutional-grade C/C++ trading platform that offers:

- **Tick-level backtesting** (10-year test in 0.3 seconds)
- **Walk-forward optimization** (12-parameter system in <25 seconds)
- **Interactive visualization** (option payoff diagrams, strategy analysis)
- **Multi-broker support** (including IBKR)
- **C/C++ scripting** (compatible with our codebase)

This document outlines three integration paths:

1. **Backtesting Integration**: Test box spread strategies on historical data
2. **Optimization Integration**: Tune strategy parameters using walk-forward analysis
3. **Visualization Integration**: Generate interactive option combo payoff diagrams

---

## 1. Backtesting Integration

### 1.1 Overview

**Current State**:

- No backtesting capability
- Only live trading or paper trading possible
- No historical validation of strategies
- Parameter selection is guesswork

**Zorro Enhancement**:

- Fast tick-level backtesting (C++ native)
- Historical data integration (can use ORATS/Massive data)
- Accurate simulation of commissions, slippage, market impact
- Visual replay of backtests with debugger

### 1.2 Architecture

```text
┌─────────────────────────────────────────┐
│   Box Spread Strategy (C++20)          │
│   - BoxSpreadStrategy class            │
│   - BoxSpreadCalculator                │
│   - Strategy logic extracted           │
└──────────────┬──────────────────────────┘
               │
       ┌───────▼───────────────────┐
       │   Zorro Backtesting API   │
       │   - Tick-level simulation │
       │   - Historical data feed  │
       │   - Commission/slippage   │
       └───────┬───────────────────┘
               │
       ┌───────▼───────────────────┐
       │   Data Sources            │
       │   - ORATS historical      │
       │   - Massive.com API       │
       │   - TWS historical data   │
       └───────────────────────────┘
```

### 1.3 Implementation Approach

#### Option A: Direct DLL Integration (Recommended)

Zorro supports DLL interface for external C++ code:

```cpp
// native/src/zorro_backtest_adapter.cpp

#include <zorro.h>
#include "box_spread_strategy.h"
#include "option_chain.h"

extern "C" {
  // Zorro callback functions
  void run() {
    // Initialize box spread strategy
    strategy::BoxSpreadStrategy strategy(...);

    // Get historical data from Zorro
    auto underlying_price = priceClose(0);
    auto option_chain = load_option_chain_from_zorro();

    // Find opportunities
    auto opportunities = strategy.find_box_spreads_in_chain(
      option_chain, underlying_price
    );

    // Execute if profitable
    for (const auto& opp : opportunities) {
      if (opp.is_actionable()) {
        enterLong(1);  // Place order via Zorro
      }
    }
  }
}
```

#### Option B: Standalone Backtesting Engine

Create a standalone backtesting engine that uses Zorro's data format:

```cpp
// native/include/backtest_engine.h
class BacktestEngine {
public:
  struct BacktestConfig {
    std::string symbol;
    std::chrono::system_clock::time_point start_date;
    std::chrono::system_clock::time_point end_date;
    double initial_capital = 100000.0;
    double commission_per_contract = 0.65;
    double slippage_per_contract = 0.01;
  };

  struct BacktestResult {
    double total_profit;
    double total_return;
    double sharpe_ratio;
    double max_drawdown;
    int total_trades;
    int winning_trades;
    double win_rate;
    std::vector<types::Position> all_trades;
  };

  BacktestResult run_backtest(
    const BacktestConfig& config,
    strategy::BoxSpreadStrategy& strategy
  );
};
```

### 1.4 Historical Data Integration

#### Using ORATS Historical Data

```cpp
// python/integration/zorro_data_adapter.py
class ZorroDataAdapter:
    """Convert ORATS historical data to Zorro format."""

    def convert_orats_to_zorro(self, symbol: str, start_date: str, end_date: str):
        """Fetch ORATS data and convert to Zorro T6 format."""

        # Fetch from ORATS
        orats_data = self.orats_client.get_historical_options(
            symbol, start_date, end_date
        )

        # Convert to Zorro T6 format (tick data)
        zorro_data = []
        for tick in orats_data:
            zorro_tick = {
                'date': self._convert_date(tick['timestamp']),
                'time': self._convert_time(tick['timestamp']),
                'open': tick['bid'],
                'high': tick['ask'],
                'low': tick['bid'],
                'close': tick['mid_price'],
                'volume': tick['volume']
            }
            zorro_data.append(zorro_tick)

        # Write to Zorro T6 file
        self._write_t6_file(symbol, zorro_data)
```

#### Using Massive.com Historical Data

Similar adapter for Massive.com API data conversion to Zorro format.

### 1.5 Configuration

Add Zorro backtesting configuration:

```json
{
  "zorro": {
    "enabled": false,
    "backtesting": {
      "enabled": true,
      "data_source": "orats", // or "massive", "tws"
      "start_date": "2020-01-01",
      "end_date": "2024-12-31",
      "initial_capital": 100000.0,
      "commission_per_contract": 0.65,
      "slippage_per_contract": 0.01,
      "tick_data": true
    }
  }
}
```

### 1.6 Usage Example

```bash

# Run backtest via Zorro

./build/bin/zorro_backtest \
  --config config/config.json \
  --symbol SPY \
  --start-date 2020-01-01 \
  --end-date 2024-12-31 \
  --output results/backtest_spy_2020_2024.json

# Or use Python wrapper

python python/integration/zorro_backtest.py \
  --symbol SPY \
  --start-date 2020-01-01 \
  --end-date 2024-12-31
```

### 1.7 Expected Benefits

- **Strategy Validation**: Test on years of historical data
- **Parameter Optimization**: Find optimal min_profit, min_roi thresholds
- **Risk Assessment**: Calculate max drawdown, Sharpe ratio
- **Performance Metrics**: Win rate, average profit per trade
- **Market Condition Testing**: Test during volatility spikes, crashes

---

## 2. Optimization Integration

### 2.1 Overview

**Current State**:

- Manual parameter selection
- No systematic optimization
- Risk of overfitting
- No robustness testing

**Zorro Enhancement**:

- Walk-forward optimization (prevents overfitting)
- Multiple optimization algorithms (genetic, brute force, ascent)
- Parameter histograms showing effect on results
- Fast optimization (<25 seconds for 12 parameters)

### 2.2 Walk-Forward Optimization (WFO)

WFO divides data into training/test cycles:

```text
Cycle 1: [Train: 2020-2021] [Test: 2022]
Cycle 2: [Train: 2020-2022] [Test: 2023]
Cycle 3: [Train: 2020-2023] [Test: 2024]
```

Each cycle:

1. Optimize parameters on training data
2. Test on out-of-sample data
3. Measure robustness

### 2.3 Optimizable Parameters

From `StrategyParams`:

```cpp
struct StrategyParams {
    double min_arbitrage_profit;      // 0.05 - 0.50 (step: 0.05)
    double min_roi_percent;           // 0.1 - 2.0 (step: 0.1)
    double max_position_size;         // 5000 - 20000 (step: 1000)
    int min_days_to_expiry;           // 20 - 45 (step: 5)
    int max_days_to_expiry;           // 45 - 90 (step: 5)
    double max_bid_ask_spread;        // 0.05 - 0.20 (step: 0.05)
    int min_volume;                   // 50 - 200 (step: 50)
    int min_open_interest;            // 250 - 1000 (step: 250)
};
```

### 2.4 Implementation

#### Zorro Script for WFO

```c
// zorro_scripts/box_spread_optimizer.c
var optimizeParameters() {
    // Define parameter ranges
    var min_profit = optimize(0.05, 0.50, 0.05, 0);
    var min_roi = optimize(0.1, 2.0, 0.1, 1);
    var min_dte = optimize(20, 45, 5, 2);
    var max_dte = optimize(45, 90, 5, 3);
    var max_spread = optimize(0.05, 0.20, 0.05, 4);

    // Load strategy from DLL
    BoxSpreadStrategyParams params;
    params.min_arbitrage_profit = min_profit;
    params.min_roi_percent = min_roi;
    params.min_days_to_expiry = min_dte;
    params.max_days_to_expiry = max_dte;
    params.max_bid_ask_spread = max_spread;

    // Run strategy
    var result = run_box_spread_strategy(params);

    // Return optimization target (Sharpe ratio)
    return result.sharpe_ratio;
}

function run() {
    // Walk-forward optimization
    NumWFOCycles = 3;
    WFOTrainDays = 365 * 2;  // 2 years training
    WFOTestDays = 365;       // 1 year testing

    // Optimize
    optimizeParameters();
}
```

#### C++ Wrapper for Zorro Optimization

```cpp
// native/src/zorro_optimizer.cpp

#include <zorro.h>
#include "box_spread_strategy.h"

extern "C" {
  double run_box_spread_strategy(BoxSpreadStrategyParams params) {
    // Initialize strategy with optimized parameters
    config::StrategyParams strategy_params;
    strategy_params.min_arbitrage_profit = params.min_arbitrage_profit;
    strategy_params.min_roi_percent = params.min_roi_percent;
    // ... set other parameters

    strategy::BoxSpreadStrategy strategy(
      nullptr,  // No live client needed
      nullptr,  // No order manager needed
      strategy_params
    );

    // Run backtest on current data window
    BacktestEngine engine;
    BacktestConfig config;
    config.start_date = WFOStartDate();
    config.end_date = WFOEndDate();

    auto result = engine.run_backtest(config, strategy);

    // Return Sharpe ratio as optimization target
    return result.sharpe_ratio;
  }
}
```

### 2.5 Optimization Algorithms

Zorro supports multiple optimization methods:

1. **Ascent** (hill climbing): Fast, good for smooth parameter spaces
2. **Brute Force**: Exhaustive search, guaranteed to find global optimum
3. **Genetic Algorithm**: Good for non-convex spaces, avoids local optima
4. **Custom Algorithm**: User-defined optimization logic

### 2.6 Configuration

```json
{
  "zorro": {
    "optimization": {
      "enabled": true,
      "method": "genetic", // or "ascent", "brute_force"
      "walk_forward": {
        "enabled": true,
        "num_cycles": 3,
        "train_days": 730,
        "test_days": 365
      },
      "parameters": {
        "min_arbitrage_profit": { "min": 0.05, "max": 0.5, "step": 0.05 },
        "min_roi_percent": { "min": 0.1, "max": 2.0, "step": 0.1 },
        "min_days_to_expiry": { "min": 20, "max": 45, "step": 5 },
        "max_days_to_expiry": { "min": 45, "max": 90, "step": 5 }
      },
      "objective": "sharpe_ratio" // or "total_return", "win_rate"
    }
  }
}
```

### 2.7 Output and Analysis

Zorro generates:

- **Parameter histograms**: Effect of each parameter on results
- **Contour charts**: 2D parameter interactions
- **Walk-forward report**: Robustness across cycles
- **Optimization report**: Best parameters found

### 2.8 Usage Example

```bash

# Run optimization

python python/integration/zorro_optimize.py \
  --config config/config.json \
  --method genetic \
  --walk-forward \
  --symbols SPY,QQQ,IWM

# View results

open results/optimization_report.html
```

### 2.9 Expected Benefits

- **Robust Parameters**: Optimized across multiple market conditions
- **Avoid Overfitting**: Walk-forward validation prevents curve-fitting
- **Data-Driven Decisions**: Empirical optimization vs. manual guessing
- **Parameter Sensitivity**: Understand which parameters matter most

---

## 3. Visualization Integration

### 3.1 Overview

**Current State**:

- Text-based logging only
- No visual payoff diagrams
- Limited strategy visualization
- No interactive analysis tools

**Zorro Enhancement**:

- Interactive option combo payoff diagrams
- Strategy performance charts
- Balance curve visualization
- Parameter effect visualization

### 3.2 Option Payoff Diagrams

Zorro can generate interactive payoff diagrams for box spreads:

```cpp
// native/src/zorro_visualizer.cpp

#include <zorro.h>
#include "box_spread_strategy.h"

class ZorroVisualizer {
public:
  void plot_box_spread_payoff(
    const types::BoxSpreadLeg& spread,
    double underlying_range_min,
    double underlying_range_max
  ) {
    // Generate payoff data points
    std::vector<PayoffPoint> points;
    for (double price = underlying_range_min;
         price <= underlying_range_max;
         price += 0.5) {

      double payoff = calculate_payoff_at_expiry(spread, price);
      points.push_back({price, payoff});
    }

    // Plot via Zorro
    plotPayoff("Box Spread Payoff", points);
  }

private:
  double calculate_payoff_at_expiry(
    const types::BoxSpreadLeg& spread,
    double underlying_price
  ) {
    // Box spread payoff is constant (strike width - net debit)
    return spread.theoretical_value - spread.net_debit;
  }
};
```

### 3.3 Strategy Performance Visualization

```cpp
void plot_strategy_performance(
  const BacktestResult& result
) {
  // Plot equity curve
  plotEquity("Strategy Equity Curve", result.equity_curve);

  // Plot drawdown
  plotDrawdown("Drawdown", result.drawdown_curve);

  // Plot trades
  plotTrades("Trade Distribution", result.trades);

  // Performance metrics
  printMetrics({
    {"Total Return", result.total_return},
    {"Sharpe Ratio", result.sharpe_ratio},
    {"Max Drawdown", result.max_drawdown},
    {"Win Rate", result.win_rate}
  });
}
```

### 3.4 Parameter Effect Visualization

Zorro's optimization generates:

- **Parameter histograms**: Effect of each parameter on Sharpe ratio
- **Contour plots**: 2D parameter interactions
- **Sensitivity analysis**: Which parameters matter most

### 3.5 Configuration

```json
{
  "zorro": {
    "visualization": {
      "enabled": true,
      "plot_payoff_diagrams": true,
      "plot_equity_curves": true,
      "plot_drawdown": true,
      "plot_trade_distribution": true,
      "interactive": true,
      "export_formats": ["png", "svg", "pdf"]
    }
  }
}
```

### 3.6 Usage Example

```bash

# Generate payoff diagram for a box spread

python python/integration/zorro_visualize.py \
  --spread-file data/spread_example.json \
  --output diagrams/box_spread_payoff.html

# Visualize backtest results

python python/integration/zorro_visualize.py \
  --backtest-results results/backtest_spy_2020_2024.json \
  --output reports/backtest_report.html
```

### 3.7 Expected Benefits

- **Visual Analysis**: Understand strategy behavior visually
- **Educational**: Better communication of strategy concepts
- **Debugging**: Identify issues through visual inspection
- **Reporting**: Professional visualizations for stakeholders

---

## 4. Implementation Roadmap

### Phase 1: Foundation (Weeks 1-2)

1. **Download and Install Zorro**

   ```bash
   # Download from https://zorro-project.com/download/
   # Extract to native/third_party/zorro/
   ```

2. **Set Up Development Environment**
   - Install Zorro SDK
   - Create Zorro script templates
   - Set up build integration

3. **Create Basic DLL Interface**
   - Implement `zorro_backtest_adapter.cpp`
   - Create Zorro C wrapper functions
   - Test basic integration

### Phase 2: Backtesting Integration (Weeks 3-4)

1. **Historical Data Adapter**
   - Implement ORATS → Zorro converter
   - Implement Massive.com → Zorro converter
   - Test data conversion accuracy

2. **Backtesting Engine**
   - Implement `BacktestEngine` class
   - Integrate with `BoxSpreadStrategy`
   - Add commission/slippage simulation

3. **Testing and Validation**
   - Run sample backtests
   - Validate results against paper trading
   - Performance benchmarking

### Phase 3: Optimization Integration (Weeks 5-6)

1. **Parameter Definition**
   - Identify optimizable parameters
   - Define parameter ranges
   - Create optimization config

2. **Walk-Forward Implementation**
   - Implement WFO cycles
   - Integrate with backtesting engine
   - Generate optimization reports

3. **Testing and Tuning**
   - Run sample optimizations
   - Validate robustness
   - Tune optimization algorithms

### Phase 4: Visualization Integration (Weeks 7-8)

1. **Payoff Diagram Generator**
   - Implement payoff calculation
   - Create Zorro plotting functions
   - Generate interactive diagrams

2. **Performance Visualization**
   - Equity curve plotting
   - Drawdown visualization
   - Trade distribution charts

3. **Integration and Testing**
   - Test visualization features
   - Generate sample reports
   - Documentation

### Phase 5: Documentation and Polish (Week 9)

1. **Documentation**
   - User guide
   - API documentation
   - Example scripts

2. **Testing**
   - End-to-end testing
   - Performance testing
   - Edge case testing

3. **Release Preparation**
   - Build scripts
   - Configuration examples
   - Installation guide

---

## 5. Code Examples

### 5.1 Backtesting Script

```cpp
// native/src/zorro_backtest_runner.cpp

#include "box_spread_strategy.h"
#include "backtest_engine.h"
#include "zorro_data_adapter.h"

int main(int argc, char* argv[]) {
  // Load configuration
  auto config = config::ConfigManager::load("config/config.json");

  // Initialize strategy
  strategy::BoxSpreadStrategy strategy(
    nullptr,  // No live client
    nullptr,  // No order manager
    config.strategy
  );

  // Set up backtest
  BacktestEngine::BacktestConfig backtest_config;
  backtest_config.symbol = "SPY";
  backtest_config.start_date = parse_date("2020-01-01");
  backtest_config.end_date = parse_date("2024-12-31");
  backtest_config.initial_capital = 100000.0;

  // Load historical data
  ZorroDataAdapter adapter;
  adapter.load_historical_data("SPY", backtest_config.start_date, backtest_config.end_date);

  // Run backtest
  BacktestEngine engine;
  auto result = engine.run_backtest(backtest_config, strategy);

  // Print results
  std::cout << "Backtest Results:\n";
  std::cout << "  Total Return: " << result.total_return << "%\n";
  std::cout << "  Sharpe Ratio: " << result.sharpe_ratio << "\n";
  std::cout << "  Max Drawdown: " << result.max_drawdown << "%\n";
  std::cout << "  Win Rate: " << result.win_rate << "%\n";
  std::cout << "  Total Trades: " << result.total_trades << "\n";

  return 0;
}
```

### 5.2 Optimization Script

```python

# python/integration/zorro_optimize.py

import json
import subprocess
from pathlib import Path

def run_zorro_optimization(config_path: str, symbol: str):
    """Run Zorro walk-forward optimization."""

    # Load config
    with open(config_path) as f:
        config = json.load(f)

    # Generate Zorro script
    zorro_script = generate_zorro_script(config, symbol)
    script_path = Path("zorro_scripts") / f"optimize_{symbol}.c"
    script_path.write_text(zorro_script)

    # Run Zorro
    result = subprocess.run([
        "zorro",
        "-script", str(script_path),
        "-optimize",
        "-train",
        "-report"
    ], capture_output=True)

    # Parse results
    results = parse_optimization_results(result.stdout)

    return results

def generate_zorro_script(config: dict, symbol: str) -> str:
    """Generate Zorro optimization script."""

    opt_config = config["zorro"]["optimization"]

    script = f"""

#include <zorro.h>

var optimizeParameters() {{
    var min_profit = optimize({opt_config["parameters"]["min_arbitrage_profit"]["min"]},
                               {opt_config["parameters"]["min_arbitrage_profit"]["max"]},
                               {opt_config["parameters"]["min_arbitrage_profit"]["step"]}, 0);
    var min_roi = optimize({opt_config["parameters"]["min_roi_percent"]["min"]},
                           {opt_config["parameters"]["min_roi_percent"]["max"]},
                           {opt_config["parameters"]["min_roi_percent"]["step"]}, 1);

    // ... more parameters ...

    BoxSpreadStrategyParams params;
    params.min_arbitrage_profit = min_profit;
    params.min_roi_percent = min_roi;

    var result = run_box_spread_strategy(params);
    return result.sharpe_ratio;
}}

function run() {{
    asset("{symbol}");
    NumWFOCycles = {opt_config["walk_forward"]["num_cycles"]};
    WFOTrainDays = {opt_config["walk_forward"]["train_days"]};
    WFOTestDays = {opt_config["walk_forward"]["test_days"]};

    optimizeParameters();
}}
"""
    return script
```

### 5.3 Visualization Script

```python

# python/integration/zorro_visualize.py

import json
import matplotlib.pyplot as plt
import numpy as np

def plot_box_spread_payoff(spread_data: dict, output_path: str):
    """Generate interactive box spread payoff diagram."""

    # Extract spread parameters
    strike_width = spread_data["strike_width"]
    net_debit = spread_data["net_debit"]

    # Calculate payoff at expiry (constant for box spread)
    payoff = strike_width - net_debit

    # Generate underlying price range
    lower_strike = spread_data["lower_strike"]
    upper_strike = spread_data["upper_strike"]
    price_range = np.linspace(lower_strike - 10, upper_strike + 10, 100)
    payoffs = np.full_like(price_range, payoff)

    # Plot
    fig, ax = plt.subplots(figsize=(12, 6))
    ax.plot(price_range, payoffs, 'b-', linewidth=2, label='Box Spread Payoff')
    ax.axhline(y=0, color='k', linestyle='--', alpha=0.3)
    ax.axvline(x=lower_strike, color='r', linestyle='--', alpha=0.3, label='Lower Strike')
    ax.axvline(x=upper_strike, color='r', linestyle='--', alpha=0.3, label='Upper Strike')

    ax.set_xlabel('Underlying Price at Expiry')
    ax.set_ylabel('Profit/Loss ($)')
    ax.set_title('Box Spread Payoff Diagram')
    ax.legend()
    ax.grid(True, alpha=0.3)

    plt.tight_layout()
    plt.savefig(output_path)
    plt.close()

def plot_backtest_results(results_path: str, output_path: str):
    """Plot backtest performance metrics."""

    with open(results_path) as f:
        results = json.load(f)

    # Extract data
    dates = [r["date"] for r in results["daily_returns"]]
    equity = [r["equity"] for r in results["daily_returns"]]

    # Create subplots
    fig, axes = plt.subplots(2, 1, figsize=(12, 10))

    # Equity curve
    axes[0].plot(dates, equity, 'b-', linewidth=2)
    axes[0].set_title('Equity Curve')
    axes[0].set_ylabel('Account Value ($)')
    axes[0].grid(True, alpha=0.3)

    # Drawdown
    drawdown = [r["drawdown"] for r in results["daily_returns"]]
    axes[1].fill_between(dates, 0, drawdown, alpha=0.3, color='r')
    axes[1].plot(dates, drawdown, 'r-', linewidth=1)
    axes[1].set_title('Drawdown')
    axes[1].set_ylabel('Drawdown (%)')
    axes[1].set_xlabel('Date')
    axes[1].grid(True, alpha=0.3)

    plt.tight_layout()
    plt.savefig(output_path)
    plt.close()
```

---

## 6. Configuration Reference

### Complete Zorro Configuration

```json
{
  "zorro": {
    "enabled": false,
    "install_path": "/opt/zorro",
    "backtesting": {
      "enabled": true,
      "data_source": "orats",
      "start_date": "2020-01-01",
      "end_date": "2024-12-31",
      "initial_capital": 100000.0,
      "commission_per_contract": 0.65,
      "slippage_per_contract": 0.01,
      "tick_data": true
    },
    "optimization": {
      "enabled": false,
      "method": "genetic",
      "walk_forward": {
        "enabled": true,
        "num_cycles": 3,
        "train_days": 730,
        "test_days": 365
      },
      "parameters": {
        "min_arbitrage_profit": { "min": 0.05, "max": 0.5, "step": 0.05 },
        "min_roi_percent": { "min": 0.1, "max": 2.0, "step": 0.1 },
        "min_days_to_expiry": { "min": 20, "max": 45, "step": 5 },
        "max_days_to_expiry": { "min": 45, "max": 90, "step": 5 },
        "max_bid_ask_spread": { "min": 0.05, "max": 0.2, "step": 0.05 }
      },
      "objective": "sharpe_ratio"
    },
    "visualization": {
      "enabled": true,
      "plot_payoff_diagrams": true,
      "plot_equity_curves": true,
      "plot_drawdown": true,
      "plot_trade_distribution": true,
      "interactive": true,
      "export_formats": ["png", "svg", "pdf"]
    }
  }
}
```

---

## 7. Dependencies and Prerequisites

### Required Software

1. **Zorro Trading Platform**
   - Download from: <https://zorro-project.com/download/>
   - Install to: `native/third_party/zorro/`
   - License: Free for personal use

2. **Zorro SDK**
   - Included with Zorro installation
   - C/C++ headers: `zorro.h`
   - DLL interface for external code

### Required Libraries

- Existing project dependencies:
  - nlohmann/json (already included)
  - spdlog (already included)
  - Catch2 (already included)

### Python Dependencies

```txt

# Add to requirements.txt

matplotlib>=3.7.0
numpy>=1.24.0
pandas>=2.0.0
plotly>=5.14.0  # For interactive plots
```

---

## 8. Testing Strategy

### Unit Tests

```cpp
// native/tests/test_zorro_backtest.cpp
TEST_CASE("Zorro backtesting adapter", "[zorro][backtest]") {
  ZorroBacktestAdapter adapter;
  BacktestConfig config;
  config.symbol = "SPY";
  config.start_date = parse_date("2020-01-01");
  config.end_date = parse_date("2020-12-31");

  auto result = adapter.run_backtest(config, strategy);

  REQUIRE(result.total_trades > 0);
  REQUIRE(result.total_return >= -100.0);  // Can't lose more than invested
  REQUIRE(result.sharpe_ratio >= -5.0);    // Reasonable bounds
}
```

### Integration Tests

```bash

# Test backtesting integration

python python/tests/test_zorro_backtest.py

# Test optimization integration

python python/tests/test_zorro_optimize.py

# Test visualization

python python/tests/test_zorro_visualize.py
```

### Validation Tests

- Compare backtest results with paper trading results
- Validate optimization results are reasonable
- Check visualization outputs are correct

---

## 9. Performance Considerations

### Backtesting Performance

- **Zorro Speed**: 10-year backtest in 0.3 seconds (C++)
- **Expected Performance**: Should match or exceed Zorro benchmarks
- **Memory Usage**: Minimal (tick-by-tick processing)

### Optimization Performance

- **Zorro Speed**: 12-parameter system in <25 seconds
- **Expected Performance**: Similar for our parameter space
- **Parallelization**: Zorro supports multi-core optimization

### Visualization Performance

- **Rendering Speed**: Real-time for interactive plots
- **File Size**: PNG < 1MB, SVG < 500KB, PDF < 2MB

---

## 10. Limitations and Considerations

### Limitations

1. **Historical Data Quality**: Depends on data source (ORATS/Massive)
2. **Slippage Modeling**: May not perfectly match live trading
3. **Commission Assumptions**: Assumes fixed commissions
4. **Market Impact**: May not fully model large order impact

### Considerations

1. **Overfitting Risk**: Always use walk-forward optimization
2. **Data Snooping**: Don't optimize on test data
3. **Market Regime Changes**: Past performance ≠ future results
4. **Implementation Risk**: Live trading may differ from backtest

---

## 11. Success Metrics

### Backtesting Integration

- ✅ Successfully run 10-year backtest on SPY
- ✅ Generate performance metrics (Sharpe, drawdown, win rate)
- ✅ Validate results match paper trading within 10%

### Optimization Integration

- ✅ Complete walk-forward optimization in <5 minutes
- ✅ Identify robust parameter sets
- ✅ Generate parameter sensitivity analysis

### Visualization Integration

- ✅ Generate interactive payoff diagrams
- ✅ Create equity curve and drawdown plots
- ✅ Export publication-quality visualizations

---

## 12. Next Steps

1. **Immediate Actions**:
   - [ ] Download Zorro and install SDK
   - [ ] Review Zorro documentation
   - [ ] Set up development environment
   - [ ] Create basic DLL interface

2. **Short Term (1-2 weeks)**:
   - [ ] Implement backtesting adapter
   - [ ] Test with sample historical data
   - [ ] Validate results

3. **Medium Term (1-2 months)**:
   - [ ] Complete backtesting integration
   - [ ] Implement optimization
   - [ ] Add visualization features

4. **Long Term**:
   - [ ] Production deployment
   - [ ] Continuous optimization
   - [ ] Regular strategy validation

---

## 13. Resources

### Zorro Documentation

- **Main Website**: <https://zorro-project.com/>
- **Manual**: <https://zorro-project.com/manual/>
- **Script Examples**: <https://zorro-project.com/scripts/>
- **Downloads**: <https://zorro-project.com/download/>

### Related Documentation

- **ORATS Integration**: `docs/ORATS_INTEGRATION.md`
- **Massive.com Integration**: `docs/MASSIVE_INTEGRATION.md`
- **Strategy Implementation**: `docs/IMPLEMENTATION_GUIDE.md`

### Community Resources

- **Zorro Forum**: <https://zorro-project.com/forum/>
- **Script Library**: <https://zorro-project.com/scripts/>
- **Video Tutorials**: Available on Zorro website

---

## Conclusion

Integrating Zorro's backtesting, optimization, and visualization capabilities will significantly enhance the IBKR box spread arbitrage system. The three integration paths complement each other:

1. **Backtesting** validates strategies on historical data
2. **Optimization** finds robust parameters systematically
3. **Visualization** provides insights and communicates results

This integration plan provides a comprehensive roadmap for implementation, with clear phases, code examples, and success metrics. The modular design allows for incremental implementation, starting with backtesting, then optimization, and finally visualization.

---

**Document Status**: Draft
**Last Updated**: 2025-11-30
**Author**: AI Assistant
**Review Status**: Pending
