// types_utils.cpp - Utility implementations for types.h that don't require TWS
#include "types.h"

namespace types {

double Position::get_market_value() const {
    return static_cast<double>(quantity) * current_price * 100.0;
}

double Position::get_cost_basis() const {
    return static_cast<double>(quantity) * avg_price * 100.0;
}

} // namespace types
