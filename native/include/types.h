// types.h - Common types and data structures for IBKR Box Spread Generator
#pragma once

#include <string>
#include <chrono>
#include <optional>
#include <vector>
#include <map>

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

enum class OptionStyle {
    European,   // Can only be exercised at expiration
    American    // Can be exercised at any time before expiration
};

// ============================================================================
// Option Contract Structure
// ============================================================================

struct OptionContract {
    std::string symbol;          // Underlying symbol (e.g., "SPY")
    std::string expiry;          // Expiration date in YYYYMMDD format
    double strike;               // Strike price
    OptionType type;             // Call or Put
    OptionStyle style;           // European or American exercise style
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

    double net_debit;             // Total cost of the spread (using mid prices)
    double theoretical_value;     // Should equal strike difference
    double arbitrage_profit;      // theoretical_value - net_debit
    double roi_percent;           // (profit / net_debit) * 100

    // Pricing details for each leg (mid prices)
    double long_call_price;
    double short_call_price;
    double long_put_price;
    double short_put_price;

    // Buy vs Sell disparity (intraday changes due to bid-ask spreads, put-call parity violations, etc.)
    // BUY: Long legs use ASK, short legs use BID (cost to buy the box)
    double buy_net_debit;         // Cost to buy box spread (ASK for long, BID for short)
    double buy_profit;            // Profit from buying box spread
    double buy_implied_rate;      // Implied rate when buying

    // SELL: Long legs use BID, short legs use ASK (credit received from selling box)
    double sell_net_credit;       // Credit received from selling box spread (BID for long, ASK for short)
    double sell_profit;           // Profit from selling box spread
    double sell_implied_rate;     // Implied rate when selling

    double buy_sell_disparity;    // Difference between buy and sell profitability
    double put_call_parity_violation;  // Put-call parity violation amount (bps)

    // Liquidity metrics
    double long_call_bid_ask_spread;
    double short_call_bid_ask_spread;
    double long_put_bid_ask_spread;
    double short_put_bid_ask_spread;

    // Margin requirements
    double initial_margin;              // Initial margin required (Reg-T or portfolio)
    double maintenance_margin;         // Maintenance margin required
    double portfolio_margin_benefit;    // Margin reduction from portfolio margining
    double reg_t_margin;                // Reg-T margin if applicable
    double span_margin;                 // SPAN margin if applicable
    bool uses_portfolio_margin;         // Whether portfolio margin is used
    std::chrono::system_clock::time_point margin_timestamp;  // When margin was calculated

    // Helper methods
    bool is_valid() const;
    double get_strike_width() const;
    int get_days_to_expiry() const;
    double get_effective_margin() const;  // Returns initial_margin or portfolio_margin if available
};

// ============================================================================
// Yield Curve Data Point
// ============================================================================

struct YieldCurvePoint {
    std::string symbol;              // Underlying symbol (e.g., "SPX", "ES", "XSP")
    int days_to_expiry;              // Days to expiration
    std::string expiry_date;         // Expiration date (YYYYMMDD)
    double strike_width;             // Strike width (K2 - K1)
    double implied_rate;             // Implied annual interest rate (%)
    double effective_rate;           // Effective rate after transaction costs (%)
    double net_debit;                // Net debit/credit
    double spread_bps;               // Spread vs benchmark (basis points)
    double liquidity_score;          // Average liquidity score across legs
    BoxSpreadLeg spread;             // Full box spread leg data

    // Timestamp
    std::chrono::system_clock::time_point timestamp;

    // Helper methods
    bool is_valid() const;
};

// ============================================================================
// Yield Curve
// ============================================================================

struct YieldCurve {
    std::string symbol;              // Underlying symbol
    double strike_width;             // Strike width for all points
    std::vector<YieldCurvePoint> points;  // Data points sorted by days_to_expiry
    double benchmark_rate;           // Benchmark rate for comparison (%)
    std::chrono::system_clock::time_point generated_time;

    // Helper methods
    bool is_valid() const;
    size_t size() const { return points.size(); }
    void sort_by_dte();  // Sort points by days to expiry
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

    // Optional daily return series for Pearson correlation in risk calculations.
    // Populate from TWS historical bar data; empty = use sign-based fallback.
    std::vector<double> historical_returns;

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

} // namespace types
