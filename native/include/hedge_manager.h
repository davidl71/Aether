// hedge_manager.h - Hedging management for box spread positions
#pragma once

#include "types.h"
#include <string>
#include <vector>
#include <optional>
#include <chrono>

// Forward declaration — avoids pulling in the full TWSClient header.
namespace tws { class TWSClient; }

namespace hedge {

// ============================================================================
// Interest Rate Futures Hedge
// ============================================================================

enum class InterestRateFutureType {
    SOFR_3M,        // SOFR 3-Month Futures (SR3)
    SOFR_1M,        // SOFR 1-Month Futures (SR1)
    Eurodollar,     // Eurodollar Futures (ED)
    Fed_Funds,      // Fed Funds Futures (ZQ)
    Treasury_Bill   // Treasury Bill Futures
};

struct InterestRateFuture {
    InterestRateFutureType type;
    std::string symbol;              // TWS symbol (e.g., "SR3", "ED")
    std::string expiry;              // Expiration date (YYYYMMDD)
    double current_price;            // Current futures price
    double contract_size;            // Contract size ($1M for SOFR/ED)
    double tick_size;                // Minimum price movement (0.0025 for SOFR)
    double tick_value;               // Dollar value per tick ($6.25 for SOFR)
    int days_to_expiry;

    // Calculate implied rate from futures price
    double calculate_implied_rate() const;

    // Calculate hedge ratio (how many futures contracts to hedge box spread)
    double calculate_hedge_ratio(
        const types::BoxSpreadLeg& box_spread,
        double box_spread_notional
    ) const;
};

// ============================================================================
// Currency Hedge
// ============================================================================

struct CurrencyHedge {
    std::string base_currency;       // Base currency (e.g., "USD")
    std::string hedge_currency;      // Hedge currency (e.g., "ILS")
    std::string pair_symbol;         // Currency pair (e.g., "USDILS")
    double current_rate;             // Current exchange rate
    double hedge_rate;               // Desired hedge rate (if different)
    double exposure_amount;          // Amount exposed in base currency
    double hedge_amount;             // Amount to hedge in hedge currency

    // Calculate hedge amount needed
    double calculate_hedge_amount(double exposure_usd) const;

    // Calculate hedge cost (cost of currency futures/forwards)
    double calculate_hedge_cost() const;
};

// ============================================================================
// Hedge Strategy
// ============================================================================

struct HedgeStrategy {
    // Interest rate hedging
    bool hedge_interest_rate = false;
    InterestRateFutureType rate_future_type = InterestRateFutureType::SOFR_3M;
    double target_hedge_ratio = 1.0;  // Full hedge (1.0 = 100%)

    // Currency hedging
    bool hedge_currency = false;
    std::string hedge_currency_code;  // Currency to hedge against (e.g., "ILS")
    double currency_hedge_ratio = 1.0;  // Full hedge

    // Hedge parameters
    double max_hedge_cost_bps = 50.0;  // Maximum hedge cost in basis points
    bool dynamic_hedging = false;      // Adjust hedge as rates move
    double rebalance_threshold_bps = 10.0;  // Rebalance if hedge moves by X bps
};

// ============================================================================
// Hedge Manager Class
// ============================================================================

class HedgeManager {
public:
    explicit HedgeManager(tws::TWSClient *client = nullptr);
    ~HedgeManager();

    // ========================================================================
    // Interest Rate Hedging
    // ========================================================================

    // Find suitable interest rate future for hedging
    std::optional<InterestRateFuture> find_rate_future_hedge(
        const types::BoxSpreadLeg& box_spread,
        InterestRateFutureType preferred_type = InterestRateFutureType::SOFR_3M
    );

    // Calculate hedge ratio and required contracts
    struct RateHedgeCalculation {
        InterestRateFuture future;
        int contracts_needed;        // Number of futures contracts
        double hedge_ratio;          // Hedge ratio (1.0 = full hedge)
        double hedge_cost;           // Cost of hedge position
        double basis_risk_bps;       // Basis risk in basis points (rate mismatch)
        bool is_valid;
    };

    RateHedgeCalculation calculate_rate_hedge(
        const types::BoxSpreadLeg& box_spread,
        double box_spread_notional,
        const InterestRateFuture& future,
        double target_hedge_ratio = 1.0
    );

    // ========================================================================
    // Currency Hedging
    // ========================================================================

    // Calculate currency hedge for box spread position
    CurrencyHedge calculate_currency_hedge(
        const types::BoxSpreadLeg& box_spread,
        const std::string& base_currency,
        const std::string& hedge_currency,
        double exposure_amount
    );

    // Get current exchange rate (requires market data)
    double get_exchange_rate(
        const std::string& base_currency,
        const std::string& hedge_currency
    ) const;

    // Calculate hedge cost for currency
    double calculate_currency_hedge_cost(
        const CurrencyHedge& hedge
    ) const;

    // ========================================================================
    // Combined Hedging
    // ========================================================================

    // Calculate complete hedge (interest rate + currency) for box spread
    struct CompleteHedge {
        RateHedgeCalculation rate_hedge;
        CurrencyHedge currency_hedge;
        double total_hedge_cost;
        double total_hedge_cost_bps;  // Cost as basis points of position
        bool is_viable;               // True if hedge cost is acceptable
    };

    CompleteHedge calculate_complete_hedge(
        const types::BoxSpreadLeg& box_spread,
        double box_spread_notional,
        const HedgeStrategy& strategy
    );

    // ========================================================================
    // Hedge Monitoring
    // ========================================================================

    // Monitor hedge effectiveness
    struct HedgeEffectiveness {
        double current_hedge_ratio;
        double target_hedge_ratio;
        double hedge_drift_bps;       // Drift from target in basis points
        bool needs_rebalance;
        double rebalance_cost;
    };

    // actual_contracts: number of futures contracts currently filled/open.
    // Pass -1 (default) when live position data is unavailable; the function
    // then uses the target contracts_needed as a proxy.
    HedgeEffectiveness monitor_hedge(
        const RateHedgeCalculation& hedge,
        const types::BoxSpreadLeg& box_spread,
        int actual_contracts = -1
    ) const;

private:
    tws::TWSClient *client_;
    // Helper methods
    double calculate_basis_risk(
        double box_spread_rate,
        double futures_implied_rate
    ) const;

    double convert_dte_to_futures_contract(
        int box_spread_dte,
        InterestRateFutureType future_type
    ) const;
};

} // namespace hedge
