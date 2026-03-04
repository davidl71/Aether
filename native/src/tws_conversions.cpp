// tws_conversions.cpp - TWS API conversion helpers and date utilities
#include "tws_conversions.h"
#include "market_hours.h"
#include <spdlog/spdlog.h>
#include <chrono>
#include <cstring>
#include <ctime>

namespace tws {

Contract convert_to_tws_contract(const types::OptionContract& contract)
{
  Contract c;
  c.symbol = contract.symbol;
  c.secType = "OPT";
  c.currency = "USD";
  c.exchange = contract.exchange;
  c.lastTradeDateOrContractMonth = contract.expiry;
  c.strike = contract.strike;
  c.right = (contract.type == types::OptionType::Call) ? "C" : "P";
  c.multiplier = "100";
  return c;
}

int calculate_dte(const std::string& expiry)
{
  if (expiry.empty() || expiry.length() != 8)
  {
    spdlog::warn("Invalid expiry format: {} (expected YYYYMMDD)", expiry);
    return 0;
  }

  try
  {
    int year = std::stoi(expiry.substr(0, 4));
    int month = std::stoi(expiry.substr(4, 2));
    int day = std::stoi(expiry.substr(6, 2));

    std::tm tm_expiry = {};
    tm_expiry.tm_year = year - 1900;
    tm_expiry.tm_mon = month - 1;
    tm_expiry.tm_mday = day;
    tm_expiry.tm_hour = 16;

    auto expiry_time = std::chrono::system_clock::from_time_t(std::mktime(&tm_expiry));
    auto now = std::chrono::system_clock::now();

    if (expiry_time <= now) return 0;

    market_hours::MarketHours mh;
    int trading_days = 0;
    auto current = now;

    while (current < expiry_time)
    {
      auto status = mh.get_market_status_at(current);
      if (status.current_session != market_hours::MarketSession::Closed)
        trading_days++;
      else if (!status.is_holiday && status.reason != "weekend")
        trading_days++;

      current += std::chrono::hours(24);
      if (trading_days > 365) break;
    }
    return trading_days;
  }
  catch (const std::exception& e)
  {
    spdlog::warn("Failed to calculate DTE for {}: {}", expiry, e.what());
    return 0;
  }
}

} // namespace tws
