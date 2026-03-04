// tws_conversions.h - TWS API conversion helpers and date utilities
#pragma once

#include "types.h"
#include "Contract.h"
#include <string>

namespace tws {

/// Convert platform OptionContract to IBKR Contract
Contract convert_to_tws_contract(const types::OptionContract& contract);

/// Calculate days to expiry (trading days) from YYYYMMDD string
int calculate_dte(const std::string& expiry);

} // namespace tws
