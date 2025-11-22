# Codebase Architecture

## Overview

This document provides a high-level architectural overview of the Synthetic Financing Platform. Use `@docs CODEBASE_ARCHITECTURE.md` in Cursor to give AI context about system design and component interactions.

**Note**: Box spreads are one strategy component of this platform. The platform provides comprehensive multi-asset financing optimization, including multi-account aggregation, cash flow modeling, opportunity simulation, and investment strategy framework.

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Application Layer                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   CLI/TUI    │  │  Python API  │  │  Web Frontend│      │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
└─────────┼─────────────────┼─────────────────┼──────────────┘
          │                 │                 │
┌─────────┼─────────────────┼─────────────────┼──────────────┐
│         │    Core Trading Engine            │               │
│  ┌──────▼───────────────────────────────────▼──────┐       │
│  │         Box Spread Strategy Engine              │       │
│  │  ┌──────────────┐  ┌──────────────┐            │       │
│  │  │ Option Chain │  │ Risk Calc    │            │       │
│  │  │  Manager     │  │  (VaR, etc)  │            │       │
│  │  └──────┬───────┘  └──────┬───────┘            │       │
│  │         │                 │                      │       │
│  │  ┌──────▼─────────────────▼───────┐            │       │
│  │  │    Order Manager               │            │       │
│  │  │  (Multi-leg order tracking)     │            │       │
│  │  └──────┬─────────────────────────┘            │       │
│  └─────────┼──────────────────────────────────────┘       │
│            │                                                │
│  ┌─────────▼────────────────────────────────────────────┐  │
│  │         TWS Client (IBKR Integration)               │  │
│  │  ┌──────────────┐  ┌──────────────┐               │  │
│  │  │  EClient     │  │  EWrapper    │               │  │
│  │  │  (Requests)  │  │  (Callbacks) │               │  │
│  │  └──────────────┘  └──────────────┘               │  │
│  └─────────┬──────────────────────────────────────────┘  │
└────────────┼──────────────────────────────────────────────┘
             │
┌────────────▼──────────────────────────────────────────────┐
│              External Services                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │ TWS/Gateway │  │  QuestDB     │  │  ORATS API   │    │
│  │  (IBKR)     │  │  (Time-series│  │  (Market Data│    │
│  │             │  │   Archive)   │  │   Fallback)   │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
└────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Box Spread Strategy Engine

**Location**: `native/src/box_spread_strategy.cpp`, `native/include/box_spread_strategy.h`

**Purpose**: Identifies and validates arbitrage opportunities in box spreads.

**Key Responsibilities**:

- Scan option chains for potential box spreads
- Calculate arbitrage profit (strike width - net debit)
- Validate minimum profit thresholds
- Filter by expiration dates and strike widths

**Key Classes**:

- `BoxSpreadStrategy`: Main strategy class
- `BoxSpread`: Represents a single box spread opportunity
- `Scenario`: Represents a potential trade scenario

**Dependencies**:

- `OptionChain`: For option data
- `RiskCalculator`: For risk assessment
- `ConfigManager`: For strategy parameters

### 2. Option Chain Manager

**Location**: `native/src/option_chain.cpp`, `native/include/option_chain.h`

**Purpose**: Manages and queries option chain data from TWS API.

**Key Responsibilities**:

- Request option chains from TWS
- Cache and organize option data
- Filter options by expiration and strike
- Provide efficient lookup of options

**Key Classes**:

- `OptionChain`: Container for option data
- `OptionContract`: Individual option contract

**Dependencies**:

- `TWSClient`: For market data requests
- `types.h`: For data structures

### 3. Risk Calculator

**Location**: `native/src/risk_calculator.cpp`, `native/include/risk_calculator.h`

**Purpose**: Calculates risk metrics and validates position limits.

**Key Responsibilities**:

- Calculate Value at Risk (VaR)
- Compute position sizing based on risk limits
- Validate against maximum exposure limits
- Calculate portfolio-level risk metrics

**Key Classes**:

- `RiskCalculator`: Main risk calculation engine
- `RiskMetrics`: Risk metric results

**Dependencies**:

- `ConfigManager`: For risk parameters
- `types.h`: For position data

### 4. Order Manager

**Location**: `native/src/order_manager.cpp`, `native/include/order_manager.h`

**Purpose**: Manages multi-leg orders and tracks execution status.

**Key Responsibilities**:

- Create multi-leg box spread orders
- Track order status and fills
- Handle partial fills
- Manage order lifecycle

**Key Classes**:

- `OrderManager`: Main order management class
- `Order`: Individual order representation
- `OrderState`: Order status tracking

**Dependencies**:

- `TWSClient`: For order submission
- `BoxSpreadStrategy`: For order creation
- `types.h`: For order data structures

### 5. TWS Client

**Location**: `native/src/tws_client.cpp`, `native/include/tws_client.h`

**Purpose**: Interface to Interactive Brokers TWS/Gateway API.

**Key Responsibilities**:

- Establish connection to TWS/Gateway
- Request market data
- Submit orders
- Handle callbacks (EWrapper)
- Manage connection state

**Key Classes**:

- `TWSClient`: Main client class (inherits from `DefaultEWrapper`)
- Uses TWS API classes: `EClientSocket`, `Contract`, `Order`

**Dependencies**:

- TWS API library (`libtwsapi.dylib`)
- Protocol Buffers (for TWS messages)
- Intel Decimal Library (for price precision)

### 6. Config Manager

**Location**: `native/src/config_manager.cpp`, `native/include/config_manager.h`

**Purpose**: Loads and validates JSON configuration.

**Key Responsibilities**:

- Parse JSON configuration file
- Validate configuration parameters
- Provide typed access to config values
- Handle configuration errors

**Key Classes**:

- `ConfigManager`: Configuration management class
- `Config`: Configuration data structure

**Dependencies**:

- nlohmann/json: For JSON parsing

## Data Flow

### 1. Initialization Flow

```
main() → ConfigManager::load()
       → TWSClient::connect()
       → OptionChain::initialize()
       → BoxSpreadStrategy::initialize()
```

### 2. Trading Flow

```
TWSClient::onTickPrice() → OptionChain::update()
                         → BoxSpreadStrategy::scan()
                         → RiskCalculator::validate()
                         → OrderManager::create_order()
                         → TWSClient::place_order()
```

### 3. Order Execution Flow

```
TWSClient::onOrderStatus() → OrderManager::update_status()
TWSClient::onExecution()   → OrderManager::record_fill()
TWSClient::onPosition()     → OrderManager::update_position()
```

## Key Data Structures

### Types (`native/include/types.h`)

**OptionContract**:

- `symbol`: Option symbol (e.g., "SPY 250120C00500000")
- `underlying`: Underlying symbol (e.g., "SPY")
- `strike`: Strike price
- `expiration`: Expiration date
- `option_type`: Call or Put
- `multiplier`: Contract multiplier (usually 100)

**BoxSpread**:

- `legs`: Array of 4 option contracts
- `net_debit`: Total cost to enter
- `strike_width`: Difference between strikes
- `profit`: Calculated arbitrage profit
- `roi`: Return on investment percentage

**Order**:

- `order_id`: TWS order ID
- `contracts`: Array of option contracts
- `quantities`: Array of quantities (positive = buy, negative = sell)
- `status`: Order status (Pending, Filled, etc.)
- `fills`: Array of fill records

## Threading Model

- **Main Thread**: Application logic, strategy execution
- **TWS Callback Thread**: EWrapper callbacks (from TWS API)
- **Order Tracking**: Thread-safe order status updates
- **Market Data**: Thread-safe option chain updates

**Synchronization**:

- Mutexes protect shared data structures
- Atomic flags for connection state
- Condition variables for waiting on callbacks

## Error Handling

### Error Categories

1. **Configuration Errors**: Invalid config → Exception → Exit
2. **Connection Errors**: TWS connection failed → Retry → Fallback
3. **Market Data Errors**: Request failed → Log → Continue
4. **Order Errors**: Order rejected → Log → Skip opportunity
5. **Risk Limit Errors**: Risk limit exceeded → Skip trade → Log

### Error Recovery

- **Transient Errors**: Automatic retry with exponential backoff
- **Permanent Errors**: Log and continue (don't crash)
- **Critical Errors**: Log, alert, graceful shutdown

## Configuration Structure

```json
{
  "tws": {
    "host": "127.0.0.1",
    "port": 7497,
    "client_id": 1
  },
  "strategy": {
    "symbols": ["SPY"],
    "min_arbitrage_profit": 10.0,
    "min_roi_percent": 5.0,
    "max_position_size": 1000.0
  },
  "risk": {
    "max_total_exposure": 10000.0,
    "max_positions": 5,
    "max_daily_loss": 500.0
  }
}
```

## Testing Strategy

### Unit Tests

- Individual component tests (Catch2)
- Mock TWS client for isolation
- Test edge cases and error conditions

### Integration Tests

- Full trading flow with mock TWS
- Configuration validation
- Risk limit enforcement

### Paper Trading Tests

- Real TWS connection (paper account)
- End-to-end validation
- Performance monitoring

## Performance Considerations

### Optimization Points

1. **Option Chain Caching**: Avoid redundant TWS requests
2. **Incremental Updates**: Only update changed options
3. **Parallel Scanning**: Scan multiple symbols concurrently
4. **Efficient Lookups**: Use hash maps for option lookup

### Resource Limits

- **Memory**: Option chain cache size limits
- **CPU**: Strategy scanning frequency
- **Network**: TWS API rate limits
- **Disk**: Log file rotation

## Security Considerations

1. **Credentials**: Never stored in code, use config file (gitignored)
2. **Dry-Run Mode**: Default to safe mode
3. **Paper Trading**: Use port 7497 for testing
4. **Input Validation**: Validate all TWS responses
5. **Error Logging**: Never log credentials

## Extension Points

### Adding New Strategies

1. Create new strategy class (inherit from base)
2. Implement `scan()` method
3. Register in main application
4. Add configuration parameters

### Adding New Data Sources

1. Create new data source interface
2. Implement TWS client integration
3. Add to option chain manager
4. Update configuration

### Adding New Risk Models

1. Extend `RiskCalculator` class
2. Implement new risk metrics
3. Add to validation pipeline
4. Update configuration

## Related Documentation

- **API Documentation**: `docs/API_DOCUMENTATION_INDEX.md`
- **TWS Integration**: `docs/TWS_INTEGRATION_STATUS.md`
- **Implementation Guide**: `docs/IMPLEMENTATION_GUIDE.md`
- **Quick Start**: `docs/QUICK_START.md`
