// tws_market_data.cpp - Market data tick callbacks and request handling
#include "tws_market_data.h"
#include "tws_connection.h"
#include "tws_conversions.h"
#include "cache_client.h"
#include <spdlog/spdlog.h>

#include "EClientSocket.h"
#include "Contract.h"
#include "TagValue.h"

#include <cfloat>
#include <sstream>
#include <string>
#include <thread>

namespace tws {

namespace {

std::string market_data_to_string(const types::MarketData& md)
{
  auto ms = std::chrono::duration_cast<std::chrono::milliseconds>(md.timestamp.time_since_epoch()).count();
  std::ostringstream os;
  os << md.symbol << "\t" << ms << "\t" << md.bid << "\t" << md.ask << "\t" << md.last
     << "\t" << md.bid_size << "\t" << md.ask_size << "\t" << md.last_size << "\t" << md.volume
     << "\t" << md.high << "\t" << md.low << "\t" << md.close << "\t" << md.open << "\t";
  if (md.implied_volatility.has_value()) os << *md.implied_volatility; else os << "n";
  os << "\t";
  if (md.delta.has_value()) os << *md.delta; else os << "n";
  os << "\t";
  if (md.gamma.has_value()) os << *md.gamma; else os << "n";
  os << "\t";
  if (md.theta.has_value()) os << *md.theta; else os << "n";
  os << "\t";
  if (md.vega.has_value()) os << *md.vega; else os << "n";
  return os.str();
}

std::optional<types::MarketData> market_data_from_string(const std::string& s)
{
  types::MarketData md;
  std::istringstream is(s);
  std::string token;
  auto next = [&]() -> bool { return static_cast<bool>(std::getline(is, token, '\t')); };
  if (!next()) return std::nullopt;
  md.symbol = token;
  if (!next()) return std::nullopt;
  int64_t ms = std::stoll(token);
  md.timestamp = std::chrono::system_clock::time_point(std::chrono::milliseconds(ms));
  auto read_d = [&]() { if (!next()) return 0.0; return std::stod(token); };
  auto read_i = [&]() { if (!next()) return 0; return std::stoi(token); };
  md.bid = read_d(); md.ask = read_d(); md.last = read_d();
  md.bid_size = read_i(); md.ask_size = read_i(); md.last_size = read_i();
  md.volume = read_d(); md.high = read_d(); md.low = read_d(); md.close = read_d(); md.open = read_d();
  if (next() && token != "n") md.implied_volatility = std::stod(token);
  if (next() && token != "n") md.delta = std::stod(token);
  if (next() && token != "n") md.gamma = std::stod(token);
  if (next() && token != "n") md.theta = std::stod(token);
  if (next() && token != "n") md.vega = std::stod(token);
  return md;
}

} // namespace

MarketDataHandler::MarketDataHandler(EClientSocket& client)
    : client_(client) {}

void MarketDataHandler::set_market_data_cache(platform::CacheClient* cache, int ttl_seconds)
{
  market_data_cache_ = cache;
  market_data_cache_ttl_ = ttl_seconds > 0 ? ttl_seconds : 60;
}

// ============================================================================
// Public interface
// ============================================================================

int MarketDataHandler::request_market_data(const types::OptionContract& contract,
                                           MarketDataCallback callback,
                                           ConnectionHandler& conn)
{
  if (conn.is_mock_mode())
  {
    int request_id = conn.claim_request_id();
    auto data = generate_mock_market_data(contract);
    {
      std::lock_guard<std::mutex> lock(mutex_);
      market_data_[request_id] = data;
      callbacks_[request_id] = callback;
    }
    std::thread([this, request_id]() {
      std::this_thread::sleep_for(std::chrono::milliseconds(50));
      MarketDataCallback cb;
      types::MarketData md;
      {
        std::lock_guard<std::mutex> lock(mutex_);
        auto it = callbacks_.find(request_id);
        if (it == callbacks_.end()) return;
        cb = it->second;
        md = market_data_[request_id];
      }
      if (cb) cb(md);
    }).detach();
    return request_id;
  }

  if (!conn.check_rate_limit())
  {
    spdlog::error("Rate limit exceeded: Cannot request market data for {}",
                  contract.to_string());
    return -1;
  }

  int request_id = conn.claim_request_id();

  if (!conn.can_start_market_data(request_id))
  {
    spdlog::error("Market data line limit exceeded for {}", contract.to_string());
    return -1;
  }

  Contract tws_contract = convert_to_tws_contract(contract);

  {
    std::lock_guard<std::mutex> lock(mutex_);
    callbacks_[request_id] = callback;
    ticker_to_symbol_[request_id] = contract.symbol;
  }

  conn.record_rate_message();
  conn.start_market_data_tracking(request_id);

  client_.reqMktData(request_id, tws_contract, "", false, false, TagValueListSPtr());

  spdlog::debug("Requested market data for {} (id={})", contract.to_string(), request_id);
  return request_id;
}

std::optional<types::MarketData> MarketDataHandler::request_market_data_sync(
    const types::OptionContract& contract, int timeout_ms,
    ConnectionHandler& conn)
{
  const std::string cache_key = "md:" + contract.to_string();
  if (market_data_cache_ && market_data_cache_->is_healthy())
  {
    auto opt = market_data_cache_->get(cache_key);
    if (opt.has_value())
    {
      auto parsed = market_data_from_string(*opt);
      if (parsed.has_value() && (parsed->bid > 0 || parsed->ask > 0 || parsed->last > 0))
      {
        spdlog::debug("Market data cache hit for {}", contract.to_string());
        return *parsed;
      }
    }
  }

  int request_id = conn.claim_request_id();

  auto promise = std::make_shared<std::promise<types::MarketData>>();
  auto future = promise->get_future();

  {
    std::lock_guard<std::mutex> lock(mutex_);
    promises_[request_id] = promise;
  }

  Contract tws_contract = convert_to_tws_contract(contract);

  if (!conn.check_rate_limit())
  {
    spdlog::error("Rate limit exceeded for sync market data: {}", contract.to_string());
    return std::nullopt;
  }

  if (!conn.can_start_market_data(request_id))
  {
    spdlog::error("Market data line limit exceeded for {}", contract.to_string());
    return std::nullopt;
  }

  conn.record_rate_message();
  conn.start_market_data_tracking(request_id);

  client_.reqMktData(request_id, tws_contract, "", false, false, TagValueListSPtr());

  if (future.wait_for(std::chrono::milliseconds(timeout_ms)) == std::future_status::ready)
  {
    auto data = future.get();
    if (data.bid > 0 || data.ask > 0 || data.last > 0)
    {
      if (market_data_cache_ && market_data_cache_->is_healthy())
        market_data_cache_->set(cache_key, market_data_to_string(data), market_data_cache_ttl_);
      return data;
    }
  }
  else
  {
    spdlog::warn("Market data request {} timed out after {}ms", request_id, timeout_ms);
    cancel_market_data(request_id, conn);
  }

  {
    std::lock_guard<std::mutex> lock(mutex_);
    promises_.erase(request_id);
  }
  return std::nullopt;
}

void MarketDataHandler::cancel_market_data(int request_id, ConnectionHandler& conn)
{
  if (conn.is_mock_mode())
  {
    std::lock_guard<std::mutex> lock(mutex_);
    callbacks_.erase(request_id);
    market_data_.erase(request_id);
    promises_.erase(request_id);
    return;
  }

  client_.cancelMktData(request_id);
  conn.end_market_data_tracking(request_id);
  conn.record_rate_message();

  std::lock_guard<std::mutex> lock(mutex_);
  callbacks_.erase(request_id);
  market_data_.erase(request_id);
  ticker_to_symbol_.erase(request_id);
  if (promises_.count(request_id))
  {
    types::MarketData empty_data;
    promises_[request_id]->set_value(empty_data);
    promises_.erase(request_id);
  }
}

std::vector<types::OptionContract> MarketDataHandler::request_option_chain(
    const std::string& symbol, const std::string& expiry,
    ConnectionHandler& conn)
{
  spdlog::debug("Requesting option chain for {} (expiry={})",
                symbol, expiry.empty() ? "all" : expiry);

  if (conn.is_mock_mode())
  {
    std::vector<types::OptionContract> contracts;
    std::vector<std::string> expiries = {
        expiry.empty() ? "20251219" : expiry, "20260116"};
    std::vector<double> strikes = {100.0, 105.0, 110.0};
    for (const auto& exp : expiries)
    {
      for (double strike : strikes)
      {
        contracts.push_back(make_mock_contract(symbol, exp, strike, types::OptionType::Call));
        contracts.push_back(make_mock_contract(symbol, exp, strike, types::OptionType::Put));
      }
    }
    return contracts;
  }

  if (!conn.is_connected())
  {
    spdlog::error("Cannot request option chain: Not connected");
    return {};
  }

  if (!conn.check_rate_limit())
  {
    spdlog::error("Rate limit exceeded for option chain: {}", symbol);
    return {};
  }

  int request_id = conn.claim_request_id();

  auto promise = std::make_shared<std::promise<std::vector<types::OptionContract>>>();
  auto future = promise->get_future();

  {
    std::lock_guard<std::mutex> lock(option_chain_mutex_);
    option_chain_promises_[request_id] = promise;
    option_chain_expirations_[request_id] = std::set<std::string>();
    option_chain_strikes_[request_id] = std::set<double>();
    option_chain_symbols_[request_id] = symbol;
    option_chain_complete_ = false;
  }

  // Get underlying conId
  Contract underlying;
  underlying.symbol = symbol;
  underlying.secType = "STK";
  underlying.currency = "USD";
  underlying.exchange = "SMART";

  int underlying_req_id = conn.claim_request_id();
  auto underlying_promise = std::make_shared<std::promise<long>>();
  auto underlying_future = underlying_promise->get_future();

  // Store the underlying contract details promise temporarily
  // (will be resolved via on_contract_details in ContractHandler, but we
  //  handle it inline here since option chain is self-contained)
  conn.record_rate_message();
  client_.reqContractDetails(underlying_req_id, underlying);

  long underlying_con_id = 0;
  if (underlying_future.wait_for(std::chrono::milliseconds(3000)) == std::future_status::ready)
  {
    underlying_con_id = underlying_future.get();
  }
  else
  {
    spdlog::warn("Underlying contract details timeout for {}, using 0", symbol);
    underlying_con_id = 0;
  }

  conn.record_rate_message();
  client_.reqSecDefOptParams(request_id, symbol, "", "STK", underlying_con_id);

  auto status = future.wait_for(std::chrono::milliseconds(10000));
  if (status == std::future_status::timeout)
  {
    spdlog::warn("Option chain request timeout for {}", symbol);
    std::lock_guard<std::mutex> lock(option_chain_mutex_);
    option_chain_promises_.erase(request_id);
    option_chain_expirations_.erase(request_id);
    option_chain_strikes_.erase(request_id);
    option_chain_symbols_.erase(request_id);
    return {};
  }

  std::vector<types::OptionContract> contracts = future.get();
  {
    std::lock_guard<std::mutex> lock(option_chain_mutex_);
    option_chain_promises_.erase(request_id);
    option_chain_expirations_.erase(request_id);
    option_chain_strikes_.erase(request_id);
    option_chain_symbols_.erase(request_id);
  }

  if (!expiry.empty())
  {
    contracts.erase(
        std::remove_if(contracts.begin(), contracts.end(),
                       [&expiry](const types::OptionContract& c) { return c.expiry != expiry; }),
        contracts.end());
  }

  spdlog::info("Retrieved {} option contracts for {}", contracts.size(), symbol);
  return contracts;
}

// ============================================================================
// EWrapper callback forwarding
// ============================================================================

void MarketDataHandler::on_tick_price(int ticker_id, int field, double price)
{
  try
  {
    std::lock_guard<std::mutex> lock(mutex_);
    auto& md = market_data_[ticker_id];

    switch (field)
    {
      case 1: md.bid = price; break;   // BID
      case 2: md.ask = price; break;   // ASK
      case 4: md.last = price; break;  // LAST
      case 6: md.high = price; break;  // HIGH
      case 7: md.low = price; break;   // LOW
      case 9: md.close = price; break; // CLOSE
      case 14: md.open = price; break; // OPEN
      default: break;
    }

    md.timestamp = std::chrono::system_clock::now();

    if (callbacks_.count(ticker_id))
    {
      callbacks_[ticker_id](md);
    }
    if (promises_.count(ticker_id))
    {
      promises_[ticker_id]->set_value(md);
      promises_.erase(ticker_id);
    }
  }
  catch (const std::exception& e)
  {
    spdlog::error("Exception in tickPrice(id={}, field={}): {}", ticker_id, field, e.what());
  }
}

void MarketDataHandler::on_tick_size(int ticker_id, int field, int64_t size)
{
  try
  {
    std::lock_guard<std::mutex> lock(mutex_);
    auto& md = market_data_[ticker_id];

    switch (field)
    {
      case 0: md.bid_size = static_cast<int>(size); break;  // BID_SIZE
      case 3: md.ask_size = static_cast<int>(size); break;  // ASK_SIZE
      case 5: md.last_size = static_cast<int>(size); break; // LAST_SIZE
      case 8: md.volume = static_cast<double>(size); break; // VOLUME
      default: break;
    }
  }
  catch (const std::exception& e)
  {
    spdlog::error("Exception in tickSize(id={}, field={}): {}", ticker_id, field, e.what());
  }
}

void MarketDataHandler::on_tick_option_computation(
    int ticker_id, int /*field*/,
    double implied_vol, double delta,
    double gamma, double vega, double theta)
{
  try
  {
    std::lock_guard<std::mutex> lock(mutex_);
    auto& md = market_data_[ticker_id];

    if (implied_vol >= 0 && implied_vol != DBL_MAX)
      md.implied_volatility = implied_vol;
    if (delta != DBL_MAX) md.delta = delta;
    if (gamma != DBL_MAX) md.gamma = gamma;
    if (vega != DBL_MAX) md.vega = vega;
    if (theta != DBL_MAX) md.theta = theta;
  }
  catch (const std::exception& e)
  {
    spdlog::error("Exception in tickOptionComputation(id={}): {}", ticker_id, e.what());
  }
}

void MarketDataHandler::on_security_definition_optional_parameter(
    int req_id, const std::string& exchange,
    int /*underlying_con_id*/, const std::string& /*trading_class*/,
    const std::string& /*multiplier*/,
    const std::set<std::string>& expirations,
    const std::set<double>& strikes)
{
  try
  {
    spdlog::debug("securityDefinitionOptionalParameter: reqId={}, exchange={}, "
                  "expirations={}, strikes={}",
                  req_id, exchange, expirations.size(), strikes.size());

    std::lock_guard<std::mutex> lock(option_chain_mutex_);
    if (option_chain_promises_.count(req_id) == 0) return;

    option_chain_expirations_[req_id].insert(expirations.begin(), expirations.end());
    option_chain_strikes_[req_id].insert(strikes.begin(), strikes.end());
  }
  catch (const std::exception& e)
  {
    spdlog::error("Exception in secDefOptParam(reqId={}): {}", req_id, e.what());
  }
}

void MarketDataHandler::on_security_definition_optional_parameter_end(int req_id)
{
  try
  {
    spdlog::debug("securityDefinitionOptionalParameterEnd: reqId={}", req_id);

    std::lock_guard<std::mutex> lock(option_chain_mutex_);
    if (option_chain_promises_.count(req_id) == 0) return;

    auto& expirations = option_chain_expirations_[req_id];
    auto& strikes = option_chain_strikes_[req_id];
    std::string symbol = option_chain_symbols_[req_id];

    std::vector<types::OptionContract> contracts;
    for (const auto& exp : expirations)
    {
      for (double strike : strikes)
      {
        types::OptionContract call;
        call.symbol = symbol;
        call.expiry = exp;
        call.strike = strike;
        call.type = types::OptionType::Call;
        call.exchange = "SMART";
        call.style = types::OptionStyle::American;
        contracts.push_back(call);

        types::OptionContract put;
        put.symbol = symbol;
        put.expiry = exp;
        put.strike = strike;
        put.type = types::OptionType::Put;
        put.exchange = "SMART";
        put.style = types::OptionStyle::American;
        contracts.push_back(put);
      }
    }

    spdlog::info("Option chain complete: {} contracts for {}", contracts.size(), symbol);
    if (option_chain_promises_.count(req_id))
    {
      option_chain_promises_[req_id]->set_value(contracts);
    }
  }
  catch (const std::exception& e)
  {
    spdlog::error("Exception in secDefOptParamEnd(reqId={}): {}", req_id, e.what());
    std::lock_guard<std::mutex> lock(option_chain_mutex_);
    if (option_chain_promises_.count(req_id))
    {
      option_chain_promises_[req_id]->set_value(std::vector<types::OptionContract>());
    }
  }
}

std::string MarketDataHandler::get_symbol_for_ticker(int ticker_id) const
{
  std::lock_guard<std::mutex> lock(mutex_);
  auto it = ticker_to_symbol_.find(ticker_id);
  if (it != ticker_to_symbol_.end()) return it->second;
  return "UNKNOWN";
}

} // namespace tws
