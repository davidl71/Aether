// types.cpp - Implementation of types.h member functions
#include "types.h"
#include "tws_client.h"
#include <algorithm>

namespace types {

std::string OptionContract::to_string() const {
    return symbol + " " + expiry + " " +
           std::to_string(strike) + " " +
           option_type_to_string(type);
}

bool OptionContract::is_valid() const {
    return !symbol.empty() &&
           !expiry.empty() &&
           strike > 0 &&
           !exchange.empty();
}

bool BoxSpreadLeg::is_valid() const {
    return long_call.is_valid() &&
           short_call.is_valid() &&
           long_put.is_valid() &&
           short_put.is_valid();
}

double BoxSpreadLeg::get_strike_width() const {
    return short_call.strike - long_call.strike;
}

int BoxSpreadLeg::get_days_to_expiry() const {
    if (!long_call.expiry.empty()) {
        return tws::calculate_dte(long_call.expiry);
    }
    return 0;
}

double BoxSpreadLeg::get_effective_margin() const {
    if (uses_portfolio_margin && span_margin > 0.0) {
        return span_margin;
    }
    if (initial_margin > 0.0) {
        return initial_margin;
    }
    return net_debit * 100.0;
}

double Position::get_market_value() const {
    return static_cast<double>(quantity) * current_price * 100.0;
}

double Position::get_cost_basis() const {
    return static_cast<double>(quantity) * avg_price * 100.0;
}

bool Order::is_active() const {
    return status == OrderStatus::Pending ||
           status == OrderStatus::Submitted ||
           status == OrderStatus::PartiallyFilled;
}

bool Order::is_complete() const {
    return status == OrderStatus::Filled ||
           status == OrderStatus::Cancelled ||
           status == OrderStatus::Rejected;
}

double Order::get_total_cost() const {
    if (filled_quantity > 0) {
        return static_cast<double>(filled_quantity) * avg_fill_price * 100.0;
    }
    return static_cast<double>(quantity) * limit_price * 100.0;
}

} // namespace types
