// types.h - Common types and data structures for IBKR Box Spread Generator
#pragma once

#include <string>
#include <chrono>
#include <optional>
#include <vector>

namespace types {

// ============================================================================
// Enumerations
// ============================================================================

enum class OptionType {
    Call,
    Put
};

enum class OrderAction {
    Buy,
    Sell
};

enum class OrderStatus {
    Pending,
    Submitted,
    Filled,
    PartiallyFilled,
    Cancelled,
    Rejected,
    Error
};

enum class TimeInForce {
    Day,        // Good for day
    GTC,        // Good till cancelled
    IOC,        // Immediate or cancel
    FOK         // Fill or kill
};

// ============================================================================
// Option Contract Structure
// ============================================================================

struct OptionContract {
    std::string symbol;          // Underlying symbol (e.g., "SPY")
    std::string expiry;          // Expiration date in YYYYMMDD format
    double strike;               // Strike price
    OptionType type;             // Call or Put
    std::string exchange;        // Exchange (e.g., "SMART", "CBOE")
    std::string local_symbol;    // Exchange-specific symbol

    // Helper methods
    std::string to_string() const;
    bool is_valid() const;
};

// ============================================================================
// Box Spread Structure
// ============================================================================

struct BoxSpreadLeg {
    OptionContract long_call;     // Long call at lower strike
    OptionContract short_call;    // Short call at higher strike
    OptionContract long_put;      // Long put at higher strike
    OptionContract short_put;     // Short put at lower strike

    double net_debit;             // Total cost of the spread
    double theoretical_value;     // Should equal strike difference
    double arbitrage_profit;      // theoretical_value - net_debit
    double roi_percent;           // (profit / net_debit) * 100

    // Pricing details for each leg
    double long_call_price;
    double short_call_price;
    double long_put_price;
    double short_put_price;

    // Liquidity metrics
    double long_call_bid_ask_spread;
    double short_call_bid_ask_spread;
    double long_put_bid_ask_spread;
    double short_put_bid_ask_spread;

    // Helper methods
    bool is_valid() const;
    double get_strike_width() const;
    int get_days_to_expiry() const;
};

// ============================================================================
// Risk Metrics
// ============================================================================

struct RiskMetrics {
    double max_loss;              // Maximum possible loss
    double max_gain;              // Maximum possible gain
    double break_even;            // Break-even point
    double probability_of_profit; // Estimated probability

    // Greeks
    double delta;                 // Rate of change with underlying
    double gamma;                 // Rate of change of delta
    double theta;                 // Time decay
    double vega;                  // Volatility sensitivity
    double rho;                   // Interest rate sensitivity

    // Additional risk measures
    double var_95;                // Value at Risk (95%)
    double expected_value;        // Expected profit/loss
};

// ============================================================================
// Position Structure
// ============================================================================

struct Position {
    OptionContract contract;
    int quantity;                 // Positive for long, negative for short
    double avg_price;             // Average entry price
    double current_price;         // Current market price
    double unrealized_pnl;        // Unrealized profit/loss
    std::chrono::system_clock::time_point entry_time;
    std::chrono::system_clock::time_point last_update;

    // Helper methods
    double get_market_value() const;
    double get_cost_basis() const;
    bool is_long() const { return quantity > 0; }
    bool is_short() const { return quantity < 0; }
};

// ============================================================================
// Order Structure
// ============================================================================

struct Order {
    int order_id;                 // Broker-assigned order ID
    OptionContract contract;
    OrderAction action;           // Buy or Sell
    int quantity;                 // Number of contracts
    double limit_price;           // Limit price (0 for market orders)
    TimeInForce tif;              // Time in force
    OrderStatus status;           // Current order status

    std::chrono::system_clock::time_point submitted_time;
    std::chrono::system_clock::time_point last_update;

    int filled_quantity;          // Number of contracts filled
    double avg_fill_price;        // Average fill price
    std::string status_message;   // Status/error message from broker

    // Helper methods
    bool is_active() const;
    bool is_complete() const;
    double get_total_cost() const;
};

// ============================================================================
// Market Data Structure
// ============================================================================

struct MarketData {
    std::string symbol;
    std::chrono::system_clock::time_point timestamp;

    double bid;
    double ask;
    double last;
    int bid_size;
    int ask_size;
    int last_size;
    double volume;

    // OHLC data
    double high;
    double low;
    double close;
    double open;

    // Option-specific data
    std::optional<double> implied_volatility;
    std::optional<double> delta;
    std::optional<double> gamma;
    std::optional<double> theta;
    std::optional<double> vega;

    // Helper methods
    double get_mid_price() const { return (bid + ask) / 2.0; }
    double get_spread() const { return ask - bid; }
    double get_spread_percent() const {
        double mid = get_mid_price();
        return mid > 0 ? (get_spread() / mid) * 100.0 : 0.0;
    }
};

// ============================================================================
// Account Information
// ============================================================================

struct AccountInfo {
    std::string account_id;
    double net_liquidation;       // Total account value
    double cash_balance;          // Available cash
    double buying_power;          // Available buying power
    double maintenance_margin;    // Maintenance margin requirement
    double initial_margin;        // Initial margin requirement

    double unrealized_pnl;        // Total unrealized P&L
    double realized_pnl;          // Total realized P&L
    double day_trades_remaining;  // PDT - day trades remaining
    double gross_position_value;  // Total gross position value

    std::chrono::system_clock::time_point last_update;
    std::chrono::system_clock::time_point timestamp;  // Alias for last_update
};

// ============================================================================
// Helper Functions
// ============================================================================

// Convert OptionType to string
inline std::string option_type_to_string(OptionType type) {
    return type == OptionType::Call ? "CALL" : "PUT";
}

// Convert OrderAction to string
inline std::string order_action_to_string(OrderAction action) {
    return action == OrderAction::Buy ? "BUY" : "SELL";
}

// Convert OrderStatus to string
inline std::string order_status_to_string(OrderStatus status) {
    switch (status) {
        case OrderStatus::Pending: return "PENDING";
        case OrderStatus::Submitted: return "SUBMITTED";
        case OrderStatus::Filled: return "FILLED";
        case OrderStatus::PartiallyFilled: return "PARTIALLY_FILLED";
        case OrderStatus::Cancelled: return "CANCELLED";
        case OrderStatus::Rejected: return "REJECTED";
        case OrderStatus::Error: return "ERROR";
        default: return "UNKNOWN";
    }
}

inline std::string time_in_force_to_string(TimeInForce tif) {
    switch (tif) {
        case TimeInForce::Day: return "DAY";
        case TimeInForce::GTC: return "GTC";
        case TimeInForce::IOC: return "IOC";
        case TimeInForce::FOK: return "FOK";
        default: return "DAY";
    }
}

// Parse string to OptionType
inline std::optional<OptionType> string_to_option_type(const std::string& str) {
    if (str == "CALL" || str == "C") return OptionType::Call;
    if (str == "PUT" || str == "P") return OptionType::Put;
    return std::nullopt;
}

} // namespace types
