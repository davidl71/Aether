// box_spread_bag.h - Box Spread as Cboe Complex Symbol (BAG)
#pragma once

#include "types.h"
#include "box_spread_strategy.h"
#include <string>
#include <vector>
#include <chrono>
#include <optional>

namespace types {

// ============================================================================
// Box Spread Bag (Cboe Complex Symbol Representation)
// ============================================================================

struct BoxSpreadBag {
    // Complex symbol identifier (e.g., "SPX 25JAN24 4500/4600 BOX")
    std::string complex_symbol;

    // Cboe-style complex symbol format
    // Format: "{UNDERLYING} {EXPIRY} {K1}/{K2} BOX"
    std::string cboe_symbol;

    // Original box spread leg data
    types::BoxSpreadLeg spread;

    // Current market data for the complex instrument
    struct BagMarketData {
        double bid;              // Current bid for the bag
        double ask;              // Current ask for the bag
        double last;             // Last traded price
        double mid;              // Mid price (bid + ask) / 2
        double spread;           // Bid-ask spread
        int bid_size;
        int ask_size;
        int volume;              // Trading volume for the bag
        std::chrono::system_clock::time_point timestamp;

        // Helper methods
        double get_mid_price() const { return (bid + ask) / 2.0; }
        double get_spread() const { return ask - bid; }
    } market_data;

    // Greeks (aggregated from individual legs)
    struct BagGreeks {
        double delta;            // Aggregate delta (should be ~0 for perfect box)
        double gamma;            // Aggregate gamma (should be ~0)
        double theta;            // Aggregate theta (time decay per day)
        double vega;             // Aggregate vega (IV sensitivity)
        double rho;              // Aggregate rho (rate sensitivity)

        // Update timestamp
        std::chrono::system_clock::time_point calculated_at;

        bool is_neutral() const {
            return std::abs(delta) < 0.01 &&
                   std::abs(gamma) < 0.01 &&
                   std::abs(vega) < 0.01;
        }
    } greeks;

    // Candle/OHLC data over time
    struct BagCandle {
        double open;             // Opening price
        double high;             // High price
        double low;              // Low price
        double close;            // Closing price
        double entry;            // Entry price (when position opened)
        double volume;           // Volume
        std::chrono::system_clock::time_point period_start;
        std::chrono::system_clock::time_point period_end;
        std::chrono::system_clock::time_point updated;

        // Helper methods
        double get_range() const { return high - low; }
        double get_change() const { return close - open; }
        double get_change_pct() const {
            return open > 0 ? ((close - open) / open) * 100.0 : 0.0;
        }
    } candle;

    // Historical candles (time series)
    std::vector<BagCandle> candle_history;  // Historical OHLC data

    // Position information (if this bag is a position)
    struct BagPosition {
        int quantity;            // Number of contracts
        double entry_price;      // Average entry price
        double current_price;    // Current mark-to-market price
        double cost_basis;       // Total cost basis
        double unrealized_pnl;   // Unrealized P&L
        double realized_pnl;     // Realized P&L (if any closed)
        std::chrono::system_clock::time_point entry_time;
    } position;

    // Pricing metrics
    double theoretical_value;    // Should equal strike width
    double net_debit;            // Net debit/credit
    double implied_rate;         // Implied interest rate (%)
    double effective_rate;       // Effective rate after commissions (%)
    int days_to_expiry;

    // Liquidity metrics
    double liquidity_score;      // 0-100 liquidity score
    double execution_probability; // Probability of successful execution

    // Timestamps
    std::chrono::system_clock::time_point created_at;
    std::chrono::system_clock::time_point last_updated;

    // Helper methods
    bool is_valid() const;
    void update_candle(double price, double volume = 0.0);
    void add_candle_to_history();
    void reset_candle();  // Start new candle period
    double get_current_pnl() const;
    double get_pnl_per_contract() const;

    // Generate Cboe-style symbol
    static std::string generate_cboe_symbol(
        const std::string& underlying,
        const std::string& expiry,
        double strike_low,
        double strike_high
    );
};

// ============================================================================
// Box Spread Bag Manager
// ============================================================================

class BoxSpreadBagManager {
public:
    // Create bag from box spread leg
    static BoxSpreadBag create_bag_from_spread(
        const types::BoxSpreadLeg& spread,
        const std::string& underlying_symbol
    );

    // Calculate Greeks for box spread bag
    static BagGreeks calculate_bag_greeks(
        const types::BoxSpreadLeg& spread,
        double underlying_price,
        double time_to_expiry,
        double volatility = 0.20,
        double risk_free_rate = 0.05
    );

    // Update bag market data
    static void update_bag_market_data(
        BoxSpreadBag& bag,
        double bid,
        double ask,
        double last = 0.0,
        int bid_size = 0,
        int ask_size = 0
    );

    // Update bag Greeks
    static void update_bag_greeks(
        BoxSpreadBag& bag,
        double underlying_price,
        double volatility = 0.20,
        double risk_free_rate = 0.05
    );

    // Update bag candle with new price
    static void update_bag_candle(
        BoxSpreadBag& bag,
        double price,
        double volume = 0.0
    );

    // Convert bag to TUI-compatible format
    // (For integration with existing TUI data structures)
    static void to_tui_format(
        const BoxSpreadBag& bag,
        // Output parameters would be passed by reference
        // This is a placeholder - actual implementation depends on TUI types
    );
};

} // namespace types
