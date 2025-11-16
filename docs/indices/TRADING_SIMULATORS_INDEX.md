# Trading Simulators & Testing Tools Index

<!--
@index: trading-simulators
@category: trading-simulators
@tags: simulator, backtesting, testing, strategy-validation, rl
@last-updated: 2025-01-27
-->

**Purpose**: Focused index of all trading simulators and testing tools for quick reference.

**Full Documentation**: See `API_DOCUMENTATION_INDEX.md` for complete details.

---

## Quick Comparison

| Simulator | Type | Focus | Best For |
|-----------|------|-------|----------|
| **QuantReplay** | Open-source | Multi-asset, order book | Strategy testing, order book simulation |
| **Stotra** | Open-source | Stocks/Crypto | UI/UX reference, multiplayer |
| **PyMarketSim** | Open-source | RL training | RL agent development |
| **MarS** | Research | Order-level simulation | Market impact analysis, RL training |

---

## Decision Tree

### Which Trading Simulator?

```
Need order book simulation?
  → Yes → QuantReplay or MarS
  → No → Continue...

Need RL training environment?
  → Yes → PyMarketSim/TradingAgents or MarS
  → No → Continue...

Need UI/UX reference?
  → Yes → Stotra
  → No → QuantReplay for strategy testing

Need market impact analysis?
  → Yes → MarS (order-level granularity)
  → No → QuantReplay
```

---

## Simulator Details

### QuantReplay
- **Best For**: Strategy testing, order book simulation
- **Key Features**: Historical data playback, order book modeling, multi-asset
- **Documentation**: `../API_DOCUMENTATION_INDEX.md#quantreplay`

### Stotra
- **Best For**: UI/UX reference, multiplayer trading
- **Key Features**: React + MERN stack, multiplayer, stocks/crypto
- **Documentation**: `../API_DOCUMENTATION_INDEX.md#stotra`

### PyMarketSim / TradingAgents
- **Best For**: RL agent development, limit order book simulation
- **Key Features**: Limit order book, agent training, deep RL
- **Documentation**: `../API_DOCUMENTATION_INDEX.md#pymarketsim`

### MarS (Market Simulation)
- **Best For**: Market impact analysis, RL training, realistic simulation
- **Key Features**: Large Market Model (LMM), order-level granularity, realistic dynamics
- **Documentation**: `../API_DOCUMENTATION_INDEX.md#mars`

---

## Use Cases

### Box Spread Strategy Testing
- **QuantReplay**: Test box spread strategies with realistic order book
- **MarS**: Analyze market impact of box spread execution

### RL Agent Development
- **PyMarketSim**: Train RL agents for box spread trading
- **MarS**: Advanced RL training with realistic market dynamics

### Backtesting
- **QuantReplay**: Historical data playback for backtesting
- **MarS**: Realistic market simulation for strategy validation

---

## See Also

- **Full Documentation**: `../API_DOCUMENTATION_INDEX.md#trading-simulators`
- **Summary**: `../API_DOCUMENTATION_SUMMARY.md`
- **NotebookLM Suggestions**: `../NOTEBOOKLM_API_DOCUMENTATION_SUGGESTIONS.md`
