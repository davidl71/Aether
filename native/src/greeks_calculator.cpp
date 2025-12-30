// greeks_calculator.cpp - Greeks calculation using QuantLib
#include "greeks_calculator.h"
#include "market_hours.h"
#include <spdlog/spdlog.h>
#include <ql/quantlib.hpp>
#include <ql/instruments/payoffs.hpp>
#include <ql/pricingengines/blackcalculator.hpp>
#include <cmath>

using namespace QuantLib;

namespace risk {

GreeksCalculator::GreeksCalculator() {
    spdlog::debug("GreeksCalculator initialized");
}

std::optional<Greeks> GreeksCalculator::calculate_option_greeks(
    const types::OptionContract& contract,
    double underlying_price,
    double option_price,
    double risk_free_rate,
    double implied_volatility) const {

    if (underlying_price <= 0.0 || option_price <= 0.0 || implied_volatility < 0.0) {
        spdlog::warn("Invalid parameters for Greeks calculation: S={}, price={}, vol={}",
                     underlying_price, option_price, implied_volatility);
        return std::nullopt;
    }

    // Calculate DTE
    market_hours::MarketHours market_hours;
    int dte = 0;
    if (!contract.expiry.empty()) {
        // Parse YYYYMMDD expiry
        if (contract.expiry.length() == 8) {
            try {
                int year = std::stoi(contract.expiry.substr(0, 4));
                int month = std::stoi(contract.expiry.substr(4, 2));
                int day = std::stoi(contract.expiry.substr(6, 2));

                std::tm tm_expiry = {};
                tm_expiry.tm_year = year - 1900;
                tm_expiry.tm_mon = month - 1;
                tm_expiry.tm_mday = day;
                tm_expiry.tm_hour = 16;
                tm_expiry.tm_min = 0;
                tm_expiry.tm_sec = 0;

                auto expiry_time = std::chrono::system_clock::from_time_t(std::mktime(&tm_expiry));
                auto now = std::chrono::system_clock::now();

                if (expiry_time > now) {
                    auto current = now;
                    while (current < expiry_time) {
                        auto status = market_hours.get_market_status_at(current);
                        if (status.current_session != market_hours::MarketSession::Closed) {
                            dte++;
                        } else if (!status.is_holiday && status.reason != "weekend") {
                            dte++;
                        }
                        current += std::chrono::hours(24);
                        if (dte > 365) break;
                    }
                }
            } catch (const std::exception& e) {
                spdlog::warn("Failed to parse expiry {}: {}", contract.expiry, e.what());
            }
        }
    }

    if (dte <= 0) {
        spdlog::warn("Invalid DTE for Greeks calculation: {}", dte);
        return std::nullopt;
    }

    double time_to_expiry = days_to_years(dte);
    if (time_to_expiry <= 0.0) {
        return std::nullopt;
    }

    try {
        // Set up QuantLib parameters
        Date today = Date::todaysDate();
        Calendar calendar = TARGET();
        DayCounter dayCounter = Actual365Fixed();

        // Create option type
        Option::Type ql_type = (contract.type == types::OptionType::Call)
            ? Option::Call : Option::Put;

        // Create payoff
        ext::shared_ptr<StrikedTypePayoff> payoff(
            new PlainVanillaPayoff(ql_type, contract.strike)
        );

        // Calculate forward price (assuming no dividends for simplicity)
        // Forward = Spot * e^(r*T)
        double forward = underlying_price * std::exp(risk_free_rate * time_to_expiry);

        // Standard deviation = volatility * sqrt(time)
        double stdDev = implied_volatility * std::sqrt(time_to_expiry);

        // Discount factor = e^(-r*T)
        double discount = std::exp(-risk_free_rate * time_to_expiry);

        // Create BlackCalculator
        BlackCalculator blackCalc(payoff, forward, stdDev, discount);

        // Calculate Greeks
        Greeks greeks{};
        greeks.delta = blackCalc.delta(underlying_price);
        greeks.gamma = blackCalc.gamma(underlying_price);
        greeks.theta = blackCalc.theta(underlying_price, risk_free_rate) / 365.0;  // Per day
        greeks.vega = blackCalc.vega(0.01) / 100.0;  // Per 1% vol change
        greeks.rho = blackCalc.rho(0.01) / 100.0;   // Per 1% rate change

        return greeks;
    } catch (const std::exception& e) {
        spdlog::warn("QuantLib Greeks calculation failed: {}", e.what());
        return std::nullopt;
    }
}

Greeks GreeksCalculator::calculate_stock_greeks(int quantity) const {
    Greeks greeks{};
    // Stocks have delta = 1.0 per share, other Greeks = 0
    greeks.delta = static_cast<double>(quantity);
    greeks.gamma = 0.0;
    greeks.theta = 0.0;
    greeks.vega = 0.0;
    greeks.rho = 0.0;
    return greeks;
}

Greeks GreeksCalculator::aggregate_greeks(
    const std::vector<types::Position>& positions,
    double underlying_price,
    double risk_free_rate,
    double implied_volatility) const {

    Greeks aggregate{};

    for (const auto& pos : positions) {
        // Check if option or stock
        bool is_option = (pos.contract.expiry.length() == 8);

        if (is_option) {
            // Calculate option Greeks
            auto greeks_opt = calculate_option_greeks(
                pos.contract,
                underlying_price,
                pos.current_price,
                risk_free_rate,
                implied_volatility
            );

            if (greeks_opt) {
                // Multiply by quantity and add to aggregate
                aggregate.delta += greeks_opt->delta * static_cast<double>(pos.quantity);
                aggregate.gamma += greeks_opt->gamma * static_cast<double>(pos.quantity);
                aggregate.theta += greeks_opt->theta * static_cast<double>(pos.quantity);
                aggregate.vega += greeks_opt->vega * static_cast<double>(pos.quantity);
                aggregate.rho += greeks_opt->rho * static_cast<double>(pos.quantity);
            }
        } else {
            // Stock position
            auto stock_greeks = calculate_stock_greeks(pos.quantity);
            aggregate.delta += stock_greeks.delta;
            // Other Greeks remain 0 for stocks
        }
    }

    return aggregate;
}

double GreeksCalculator::days_to_years(int days) const {
    return static_cast<double>(days) / 365.0;
}

} // namespace risk
