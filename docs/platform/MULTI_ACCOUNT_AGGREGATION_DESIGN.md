# Multi-Account Aggregation System Design

**Version:** 1.0.0
**Last Updated:** 2025-11-18
**Status:** Design Document
**Related Tasks:** T-75, T-78, T-79

## Overview

This document designs a comprehensive system for aggregating positions and account data from multiple sources into a unified portfolio view. The system supports 21+ accounts across multiple broker types, with account-level tracking and optional portfolio-level aggregation.

## Requirements

### User Specifications

**Account Inventory:**

- **US Brokers:** 8 accounts
  - IBKR: 1 live + 1 paper
  - Alpaca: 1 live + 1 paper
  - Tradier: 1 live + 1 paper
  - Tastytrade: 1 live + 1 paper
- **Israeli Banks:** 2 accounts
  - Fibi: Market securities + cash/loans
  - Discount: Market securities + cash/loans
- **Israeli Brokers:** 2 accounts (live)
  - Meitav: Cache, margin, TASE + US instruments
  - IBI: Cache, margin, TASE + US instruments
- **Pension Funds:** 9 accounts
- **Total:** 21+ accounts

**Position Handling:**

- **Primary:** Account-level tracking (each account maintains separate position view)
- **Optional:** Portfolio-level aggregation for entire portfolio view
- **Instrument Types:** TASE securities, US instruments, cash, loans, derivatives

## Architecture

### System Components

```
┌─────────────────────────────────────────────────────────────┐
│              Portfolio Aggregator (Optional)                │
│  - Aggregates all account positions                         │
│  - Currency conversion (ILS → USD)                          │
│  - Position deduplication                                   │
│  - Unified portfolio view                                   │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       │ Uses
                       │
┌──────────────────────▼──────────────────────────────────────┐
│                  Account Manager                              │
│  - Account configuration                                    │
│  - Connection management                                    │
│  - Account status tracking                                  │
│  - Account-level position views                             │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       │ Manages
                       │
        ┌──────────────┼──────────────┬──────────────┐
        │              │              │              │
┌───────▼──────┐ ┌────▼──────┐ ┌────▼──────┐ ┌────▼──────┐
│ IBKR        │ │ Alpaca    │ │ Tradier   │ │ Tastytrade│
│ Connector   │ │ Connector │ │ Connector │ │ Connector │
└───────┬──────┘ └────┬──────┘ └────┬──────┘ └────┬──────┘
        │              │              │              │
        └──────────────┼──────────────┼──────────────┘
                       │
        ┌──────────────┼──────────────┐
        │              │              │
┌───────▼──────┐ ┌────▼──────┐ ┌────▼──────┐
│ Israeli     │ │ Israeli   │ │ Pension   │
│ Bank        │ │ Broker    │ │ Fund      │
│ Connector   │ │ Connector │ │ Connector │
└─────────────┘ └───────────┘ └───────────┘
```

## Data Model

### Account Configuration

```cpp
// native/include/ib_box_spread/account_config.h

namespace ib_box_spread {

enum class AccountType {
    IBKR_LIVE,
    IBKR_PAPER,
    ALPACA_LIVE,
    ALPACA_PAPER,
    TRADIER_LIVE,
    TRADIER_PAPER,
    TASTYTRADE_LIVE,
    TASTYTRADE_PAPER,
    ISRAELI_BANK_FIBI,
    ISRAELI_BANK_DISCOUNT,
    ISRAELI_BROKER_MEITAV,
    ISRAELI_BROKER_IBI,
    PENSION_FUND
};

enum class ConnectionType {
    TWS_API,              // IBKR TWS API (socket)
    REST_API,             // Alpaca, Tradier, Tastytrade
    EXCEL_RTD,            // Israeli broker Excel RTD
    EXCEL_DDE,            // Israeli broker Excel DDE
    WEB_SCRAPING,         // Israeli broker web scraping
    FILE_IMPORT,          // Static file import (CSV/Excel)
    CLIENT_PORTAL_API     // IBKR Client Portal API
};

struct AccountConfig {
    // Identification
    std::string account_id;           // Unique identifier
    std::string account_name;          // Display name
    AccountType account_type;
    ConnectionType connection_type;

    // Connection Details
    std::string host;                 // For TWS API
    int port;                          // For TWS API
    std::string api_key;              // For REST APIs
    std::string api_secret;           // For REST APIs
    std::string base_url;             // For REST APIs
    bool paper_trading;                // Paper trading flag

    // Israeli Broker Specific
    std::string excel_file_path;      // For Excel RTD/DDE
    std::string web_scraping_url;     // For web scraping
    std::string import_file_path;     // For file import

    // Status
    bool enabled;
    int priority;                     // Connection priority (0 = highest)
    std::chrono::system_clock::time_point last_sync;

    // Currency
    std::string base_currency;        // ILS, USD, etc.
    double currency_rate_to_usd;      // Exchange rate to USD
};

struct AccountStatus {
    std::string account_id;
    bool connected;
    bool syncing;
    std::chrono::system_clock::time_point last_successful_sync;
    std::chrono::system_clock::time_point last_error;
    std::string last_error_message;
    int consecutive_failures;
};
```

### Account-Level Position View

```cpp
// native/include/ib_box_spread/account_position_view.h

struct AccountPositionView {
    std::string account_id;
    std::string account_name;
    AccountType account_type;

    // Positions
    std::vector<types::Position> positions;

    // Account Values
    double total_value_usd;
    double cash_balance_usd;
    double margin_used_usd;
    double buying_power_usd;

    // Metadata
    std::chrono::system_clock::time_point last_update;
    bool is_stale;  // True if data is older than threshold
};
```

### Aggregated Portfolio View

```cpp
// native/include/ib_box_spread/portfolio_aggregator.h

struct AggregatedPortfolio {
    // Account-Level Views
    std::vector<AccountPositionView> account_views;

    // Aggregated Positions (optional)
    std::vector<AggregatedPosition> aggregated_positions;

    // Total Portfolio Values
    double total_portfolio_value_usd;
    double total_cash_usd;
    double total_margin_used_usd;
    double total_buying_power_usd;
    double total_loan_liabilities_usd;  // From loan manager

    // Net Portfolio Value
    double net_portfolio_value_usd;  // Assets - Liabilities

    // Currency Breakdown
    std::unordered_map<std::string, double> currency_breakdown;  // ILS, USD, etc.

    // Metadata
    std::chrono::system_clock::time_point last_aggregation;
    int accounts_connected;
    int accounts_total;
};

struct AggregatedPosition {
    std::string symbol;
    std::string exchange;  // TASE, NASDAQ, NYSE, etc.
    int total_quantity;    // Sum across all accounts
    double avg_price;
    double current_price;
    double total_value_usd;

    // Account Breakdown
    struct AccountHolding {
        std::string account_id;
        int quantity;
        double avg_price;
    };
    std::vector<AccountHolding> account_holdings;
};
```

## Account Manager

```cpp
// native/include/ib_box_spread/account_manager.h

class AccountManager {
public:
    // Initialization
    bool initialize(const std::string& config_file_path);

    // Account Configuration
    bool add_account(const AccountConfig& config);
    bool update_account(const std::string& account_id, const AccountConfig& config);
    bool remove_account(const std::string& account_id);
    std::optional<AccountConfig> get_account(const std::string& account_id) const;
    std::vector<AccountConfig> get_all_accounts() const;
    std::vector<AccountConfig> get_enabled_accounts() const;

    // Connection Management
    bool connect_account(const std::string& account_id);
    bool disconnect_account(const std::string& account_id);
    bool connect_all_enabled();
    void disconnect_all();

    // Status
    AccountStatus get_account_status(const std::string& account_id) const;
    std::vector<AccountStatus> get_all_account_statuses() const;

    // Position Retrieval
    AccountPositionView get_account_positions(const std::string& account_id);
    std::vector<AccountPositionView> get_all_account_positions();

    // Synchronization
    bool sync_account(const std::string& account_id);
    bool sync_all_accounts();
    void start_auto_sync(int interval_seconds);
    void stop_auto_sync();

private:
    std::unordered_map<std::string, AccountConfig> accounts_;
    std::unordered_map<std::string, AccountStatus> account_statuses_;
    std::unordered_map<std::string, std::unique_ptr<AccountConnector>> connectors_;
    mutable std::shared_mutex accounts_mutex_;

    // Connection factory
    std::unique_ptr<AccountConnector> create_connector(const AccountConfig& config);
};
```

## Account Connector Interface

```cpp
// native/include/ib_box_spread/account_connector.h

class AccountConnector {
public:
    virtual ~AccountConnector() = default;

    // Connection
    virtual bool connect() = 0;
    virtual void disconnect() = 0;
    virtual bool is_connected() const = 0;

    // Position Retrieval
    virtual std::vector<types::Position> get_positions() = 0;
    virtual AccountPositionView get_account_view() = 0;

    // Account Info
    virtual double get_cash_balance() = 0;
    virtual double get_total_value() = 0;
    virtual std::string get_base_currency() = 0;

    // Status
    virtual AccountStatus get_status() const = 0;
};
```

### Connector Implementations

```cpp
// IBKR TWS Connector
class IBKRTWSConnector : public AccountConnector {
    // Uses existing TWSClient
    // Supports multiple accounts via different ports
};

// Alpaca REST Connector
class AlpacaConnector : public AccountConnector {
    // Uses Alpaca REST API
    // Supports paper and live accounts
};

// Tradier REST Connector
class TradierConnector : public AccountConnector {
    // Uses Tradier REST API
};

// Tastytrade REST Connector
class TastytradeConnector : public AccountConnector {
    // Uses Tastytrade REST API
};

// Israeli Bank Connector (Fibi, Discount)
class IsraeliBankConnector : public AccountConnector {
    // Uses Excel RTD/DDE or web scraping
    // Handles market securities + cash/loans
};

// Israeli Broker Connector (Meitav, IBI)
class IsraeliBrokerConnector : public AccountConnector {
    // Uses Excel RTD/DDE or web scraping
    // Handles TASE + US instruments, cache, margin
};

// Pension Fund Connector
class PensionFundConnector : public AccountConnector {
    // Uses file import or web scraping
    // Periodic updates (typically monthly)
};
```

## Portfolio Aggregator

```cpp
// native/include/ib_box_spread/portfolio_aggregator.h

class PortfolioAggregator {
public:
    PortfolioAggregator(AccountManager& account_manager);

    // Account-Level Views (Primary)
    std::vector<AccountPositionView> get_account_views();
    AccountPositionView get_account_view(const std::string& account_id);

    // Portfolio Aggregation (Optional)
    AggregatedPortfolio get_aggregated_portfolio(bool include_aggregated_positions = false);

    // Position Aggregation
    std::vector<AggregatedPosition> aggregate_positions();

    // Currency Conversion
    double convert_to_usd(double amount, const std::string& from_currency);
    void update_currency_rates();

    // Deduplication Strategy
    enum class DeduplicationStrategy {
        MERGE,           // Merge quantities of same symbol
        KEEP_SEPARATE,   // Keep separate by account
        USER_CONFIGURED  // User-defined rules
    };

    void set_deduplication_strategy(DeduplicationStrategy strategy);

private:
    AccountManager& account_manager_;
    DeduplicationStrategy dedup_strategy_;
    std::unordered_map<std::string, double> currency_rates_;  // Currency -> USD rate

    // Position merging logic
    AggregatedPosition merge_positions(
        const std::vector<types::Position>& positions
    );
};
```

## Configuration

```json
// config/accounts.json

{
  "version": "1.0",
  "last_updated": "2025-11-18T12:00:00Z",
  "accounts": [
    {
      "account_id": "IBKR-LIVE-001",
      "account_name": "IBKR Live Trading",
      "account_type": "IBKR_LIVE",
      "connection_type": "TWS_API",
      "host": "127.0.0.1",
      "port": 7496,
      "paper_trading": false,
      "enabled": true,
      "priority": 1,
      "base_currency": "USD",
      "currency_rate_to_usd": 1.0
    },
    {
      "account_id": "IBKR-PAPER-001",
      "account_name": "IBKR Paper Trading",
      "account_type": "IBKR_PAPER",
      "connection_type": "TWS_API",
      "host": "127.0.0.1",
      "port": 7497,
      "paper_trading": true,
      "enabled": true,
      "priority": 2,
      "base_currency": "USD",
      "currency_rate_to_usd": 1.0
    },
    {
      "account_id": "ALPACA-LIVE-001",
      "account_name": "Alpaca Live",
      "account_type": "ALPACA_LIVE",
      "connection_type": "REST_API",
      "api_key": "${ALPACA_API_KEY}",
      "api_secret": "${ALPACA_API_SECRET}",
      "base_url": "https://api.alpaca.markets",
      "paper_trading": false,
      "enabled": true,
      "priority": 3,
      "base_currency": "USD",
      "currency_rate_to_usd": 1.0
    },
    {
      "account_id": "FIBI-001",
      "account_name": "Fibi Bank",
      "account_type": "ISRAELI_BANK_FIBI",
      "connection_type": "EXCEL_RTD",
      "excel_file_path": "/path/to/fibi_positions.xlsx",
      "enabled": true,
      "priority": 10,
      "base_currency": "ILS",
      "currency_rate_to_usd": 0.27
    },
    {
      "account_id": "MEITAV-001",
      "account_name": "Meitav Broker",
      "account_type": "ISRAELI_BROKER_MEITAV",
      "connection_type": "WEB_SCRAPING",
      "web_scraping_url": "https://meitav.co.il/positions",
      "enabled": true,
      "priority": 11,
      "base_currency": "ILS",
      "currency_rate_to_usd": 0.27
    }
    // ... additional accounts
  ],
  "aggregation": {
    "enabled": true,
    "deduplication_strategy": "MERGE",
    "currency_base": "USD",
    "auto_update_currency_rates": true,
    "currency_rate_update_frequency_hours": 24,
    "sync_interval_seconds": 300,
    "stale_data_threshold_seconds": 600
  }
}
```

## Integration with Investment Strategy Framework

```cpp
// native/src/portfolio_calculator.cpp (updates)

class PortfolioCalculator {
public:
    // New: Multi-account portfolio calculation
    double calculate_net_portfolio_value(
        const AccountManager& account_manager,
        const LoanManager& loan_manager,
        const PortfolioAggregator& aggregator
    ) {
        // Get aggregated portfolio (optional aggregation)
        auto portfolio = aggregator.get_aggregated_portfolio(false);  // Account-level only

        // Calculate total assets from all accounts
        double total_assets = portfolio.total_portfolio_value_usd;

        // Get loan liabilities
        double total_loan_liabilities = loan_manager.get_total_loan_liabilities_usd(
            portfolio.currency_breakdown["ILS"]  // ILS/USD rate
        );

        return total_assets - total_loan_liabilities;
    }

    // Account-level allocation
    void calculate_account_allocations(
        const AccountManager& account_manager,
        const LoanManager& loan_manager
    ) {
        auto accounts = account_manager.get_all_enabled_accounts();

        for (const auto& account : accounts) {
            auto account_view = account_manager.get_account_positions(account.account_id);

            // Calculate allocation for this account
            // ... allocation logic
        }
    }
};
```

## Implementation Phases

### Phase 1: Account Manager Core (T-78)

- Account configuration system
- Account connector interface
- Basic connection management
- Account status tracking

### Phase 2: Connector Implementations (T-78)

- IBKR TWS connector (extend existing)
- Alpaca REST connector
- Tradier REST connector (future)
- Tastytrade REST connector (future)
- Israeli broker connectors (extend existing import system)

### Phase 3: Portfolio Aggregation (T-79)

- Position aggregation logic
- Currency conversion
- Deduplication strategies
- Optional portfolio-level views

## Performance Considerations

### Connection Pooling

- Limit concurrent connections (e.g., max 10 simultaneous)
- Queue connection requests
- Prioritize high-priority accounts

### Caching

- Cache account positions (5-minute TTL)
- Cache currency rates (24-hour TTL)
- Cache account status (1-minute TTL)

### Async Operations

- Async account synchronization
- Non-blocking position retrieval
- Background currency rate updates

## Error Handling

### Connection Failures

- Retry logic with exponential backoff
- Fallback to cached data
- Partial portfolio view (available accounts only)

### Data Staleness

- Mark stale accounts (>10 minutes old)
- Alert on stale data
- Continue with available data

## Testing Requirements

### Unit Tests

- Account configuration management
- Position aggregation logic
- Currency conversion
- Deduplication strategies

### Integration Tests

- Multi-account connection
- Position synchronization
- Aggregation accuracy
- Error handling scenarios

## References

- [Multi-Broker Architecture Design](../research/architecture/MULTI_BROKER_ARCHITECTURE_DESIGN.md)
- [Alpaca API Integration Design](../research/integration/ALPACA_API_INTEGRATION_DESIGN.md)
- [Israeli Broker Position Import](../../docs/ISRAELI_BROKER_POSITION_IMPORT.md)
- [Investment Strategy Framework](INVESTMENT_STRATEGY_FRAMEWORK.md)
