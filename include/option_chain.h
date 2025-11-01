// option_chain.h - Option chain data structures and analysis
#pragma once

#include "types.h"
#include <vector>
#include <map>
#include <optional>
#include <string>

namespace option_chain {

// ============================================================================
// Option Chain Entry
// ============================================================================

struct OptionChainEntry {
    types::OptionContract contract;
    types::MarketData market_data;

    // Additional option metrics
    int open_interest = 0;
    int volume = 0;
    double theoretical_price = 0.0;
    double intrinsic_value = 0.0;
    double extrinsic_value = 0.0;
    double moneyness = 0.0;  // Strike / Spot

    // Liquidity score (0-100)
    double liquidity_score = 0.0;

    bool is_valid() const;
    bool meets_liquidity_requirements(int min_volume, int min_oi) const;
};

// ============================================================================
// Strike Chain (Calls and Puts at same strike)
// ============================================================================

struct StrikeChain {
    double strike;
    std::optional<OptionChainEntry> call;
    std::optional<OptionChainEntry> put;

    bool has_both() const { return call.has_value() && put.has_value(); }
    double get_call_iv() const;
    double get_put_iv() const;
    double get_iv_skew() const;  // Difference between put and call IV
};

// ============================================================================
// Expiry Chain (All strikes for a given expiry)
// ============================================================================

class ExpiryChain {
public:
    ExpiryChain(const std::string& symbol, const std::string& expiry);

    // Add option to chain
    void add_option(const OptionChainEntry& entry);

    // Getters
    std::string get_symbol() const { return symbol_; }
    std::string get_expiry() const { return expiry_; }
    int get_days_to_expiry() const;

    // Get all strikes
    std::vector<double> get_strikes() const;

    // Get strike chain
    std::optional<StrikeChain> get_strike_chain(double strike) const;

    // Get option by strike and type
    std::optional<OptionChainEntry> get_option(
        double strike,
        types::OptionType type
    ) const;

    // Find strikes in range
    std::vector<double> get_strikes_in_range(
        double min_strike,
        double max_strike
    ) const;

    // Find ATM (at-the-money) strike
    std::optional<double> find_atm_strike(double underlying_price) const;

    // Get all options
    std::vector<OptionChainEntry> get_all_options() const;

    // Get calls only
    std::vector<OptionChainEntry> get_calls() const;

    // Get puts only
    std::vector<OptionChainEntry> get_puts() const;

    // Filter by criteria
    std::vector<OptionChainEntry> filter_by_liquidity(
        int min_volume,
        int min_open_interest
    ) const;

    std::vector<OptionChainEntry> filter_by_moneyness(
        double min_moneyness,
        double max_moneyness
    ) const;

private:
    std::string symbol_;
    std::string expiry_;
    std::map<double, StrikeChain> strikes_;
};

// ============================================================================
// Full Option Chain (All expiries)
// ============================================================================

class OptionChain {
public:
    explicit OptionChain(const std::string& symbol);

    // Add option to chain
    void add_option(const OptionChainEntry& entry);

    // Get symbol
    std::string get_symbol() const { return symbol_; }

    // Get all expiry dates
    std::vector<std::string> get_expiries() const;

    // Get chain for specific expiry
    std::optional<ExpiryChain> get_expiry_chain(
        const std::string& expiry
    ) const;

    // Find expiries in DTE range
    std::vector<std::string> get_expiries_in_dte_range(
        int min_dte,
        int max_dte
    ) const;

    // Get all options across all expiries
    std::vector<OptionChainEntry> get_all_options() const;

    // Update underlying price (for moneyness calculations)
    void set_underlying_price(double price);
    double get_underlying_price() const { return underlying_price_; }

    // Chain statistics
    int get_total_option_count() const;
    int get_expiry_count() const;

private:
    std::string symbol_;
    double underlying_price_ = 0.0;
    std::map<std::string, ExpiryChain> expiries_;
};

// ============================================================================
// Option Chain Builder
// ============================================================================

class OptionChainBuilder {
public:
    static OptionChain build_from_market_data(
        const std::string& symbol,
        const std::vector<types::OptionContract>& contracts,
        const std::map<std::string, types::MarketData>& market_data
    );

    static void calculate_metrics(
        OptionChainEntry& entry,
        double underlying_price,
        double risk_free_rate = 0.05
    );

    // Calculate implied volatility using Black-Scholes
    static std::optional<double> calculate_implied_volatility(
        double option_price,
        double underlying_price,
        double strike,
        double time_to_expiry,
        double risk_free_rate,
        types::OptionType option_type
    );

    // Calculate theoretical option price using Black-Scholes
    static double calculate_theoretical_price(
        double underlying_price,
        double strike,
        double time_to_expiry,
        double volatility,
        double risk_free_rate,
        types::OptionType option_type
    );

    // Calculate Greeks
    static double calculate_delta(
        double underlying_price,
        double strike,
        double time_to_expiry,
        double volatility,
        double risk_free_rate,
        types::OptionType option_type
    );

    static double calculate_gamma(
        double underlying_price,
        double strike,
        double time_to_expiry,
        double volatility,
        double risk_free_rate
    );

    static double calculate_theta(
        double underlying_price,
        double strike,
        double time_to_expiry,
        double volatility,
        double risk_free_rate,
        types::OptionType option_type
    );

    static double calculate_vega(
        double underlying_price,
        double strike,
        double time_to_expiry,
        double volatility,
        double risk_free_rate
    );

private:
    // Helper: Standard normal CDF
    static double standard_normal_cdf(double x);

    // Helper: Standard normal PDF
    static double standard_normal_pdf(double x);

    // Helper: d1 and d2 for Black-Scholes
    static std::pair<double, double> calculate_d1_d2(
        double underlying_price,
        double strike,
        double time_to_expiry,
        double volatility,
        double risk_free_rate
    );
};

} // namespace option_chain
