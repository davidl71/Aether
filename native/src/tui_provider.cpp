// tui_provider.cpp - Data provider implementations
#include "tui_provider.h"
#include <spdlog/spdlog.h>
#include <nlohmann/json.hpp>
#include <sstream>
#include <random>
#include <cmath>
#include <algorithm>
#include <iomanip>
#include <ctime>

namespace tui {

// ============================================================================
// MockProvider Implementation
// ============================================================================

MockProvider::MockProvider() {
  // Default to European-style symbols: SPX, XSP, NDX (European exercise style)
  // American-style symbols (SPY, QQQ, IWM) are hidden by default
  symbols_ = {"SPX", "XSP", "NDX"};
  rand_state_ = std::chrono::system_clock::now().time_since_epoch().count();
  if (rand_state_ == 0) rand_state_ = 1;
}

MockProvider::~MockProvider() {
  Stop();
}

void MockProvider::Start() {
  if (running_.exchange(true)) {
    return;  // Already running
  }

  worker_ = std::thread(&MockProvider::GenerateLoop, this);
  spdlog::info("MockProvider started");
}

void MockProvider::Stop() {
  if (!running_.exchange(false)) {
    return;  // Already stopped
  }

  if (worker_.joinable()) {
    worker_.join();
  }
  spdlog::info("MockProvider stopped");
}

Snapshot MockProvider::GetSnapshot() {
  // Non-blocking: just return latest snapshot atomically
  std::lock_guard<std::mutex> lock(mutex_);
  return latest_snapshot_;
}

bool MockProvider::IsRunning() const {
  return running_.load();
}

void MockProvider::AddSymbol(const std::string& symbol) {
  std::lock_guard<std::mutex> lock(mutex_);
  auto it = std::find(symbols_.begin(), symbols_.end(), symbol);
  if (it == symbols_.end()) {
    symbols_.push_back(symbol);
  }
}

void MockProvider::GenerateLoop() {
  while (running_.load()) {
    auto snapshot = GenerateSnapshot();
    {
      std::lock_guard<std::mutex> lock(mutex_);
      latest_snapshot_ = snapshot;
    }
    std::this_thread::sleep_for(interval_);
  }
}

double MockProvider::RandFloat() const {
  rand_state_ = rand_state_ * 6364136223846793005ULL + 1;
  return static_cast<double>((rand_state_ >> 11) & ((1ULL << 53) - 1)) / (1ULL << 53);
}

double MockProvider::BasePriceForSymbol(const std::string& symbol) const {
  // European-style symbols
  if (symbol == "SPX") return 5090.0;  // SPX is ~10x SPY price
  if (symbol == "XSP") return 509.0;   // XSP is 1/10th SPX
  if (symbol == "NDX") return 16500.0; // Nasdaq 100 Index
  if (symbol == "RUT") return 2000.0;  // Russell 2000 Index (mixed style)

  // American-style symbols (for reference, but hidden by default)
  if (symbol == "SPY") return 509.0;
  if (symbol == "QQQ") return 389.0;
  if (symbol == "IWM") return 201.0;

  return 100.0 + RandFloat() * 50.0;
}

Snapshot MockProvider::GenerateSnapshot() {
  auto now = std::chrono::system_clock::now();
  std::vector<std::string> symbols;
  {
    std::lock_guard<std::mutex> lock(mutex_);
    symbols = symbols_;
  }

  Snapshot snapshot;
  snapshot.generated_at = now;
  snapshot.mode = "DRY-RUN";
  snapshot.strategy = "RUNNING";
  snapshot.account_id = "DU123456";

  snapshot.metrics.net_liq = 100500.0 + RandFloat() * 500.0;
  snapshot.metrics.buying_power = 80400.0 + RandFloat() * 400.0;
  snapshot.metrics.excess_liquidity = 25000.0 + RandFloat() * 1000.0;
  snapshot.metrics.margin_requirement = 15000.0 + RandFloat() * 500.0;
  snapshot.metrics.commissions = 123.45 + RandFloat() * 5.0;
  snapshot.metrics.portal_ok = true;
  snapshot.metrics.tws_ok = true;
  snapshot.metrics.orats_ok = true;
  snapshot.metrics.questdb_ok = true;

  for (const auto& sym : symbols) {
    double base = BasePriceForSymbol(sym);
    snapshot.symbols.push_back(MockSymbol(sym, base, now));
  }

  // Use European-style symbols for positions
  snapshot.positions.push_back(MockPosition("SPX BOX 675/670", 1, now));
  snapshot.positions.push_back(MockPosition("XSP BOX 390/385", 2, now));

  snapshot.historic.push_back(MockPosition("SPX BOX 670/665", 0, now - std::chrono::hours(2)));
  snapshot.historic.push_back(MockPosition("XSP BOX 395/390", 0, now - std::chrono::hours(4)));

  snapshot.orders.push_back({
    now - std::chrono::minutes(1),
    "SPX BOX submitted @ 160.00",
    "info"
  });
  snapshot.orders.push_back({
    now - std::chrono::seconds(50),
    "SPX BOX fill 1/4 added liquidity",
    "success"
  });

  snapshot.alerts.push_back({
    now - std::chrono::seconds(30),
    "Combo quote missing for SPX – using ORATS fallback",
    "warn"
  });
  snapshot.alerts.push_back({
    now - std::chrono::seconds(10),
    "Portal summary net_liq=100,523 buying_power=80,412",
    "info"
  });

  snapshot.history = MockHistory(now);
  snapshot.yield_curve = MockYieldCurve(now);
  snapshot.faqs = MockFAQs();

  return snapshot;
}

SymbolSnapshot MockProvider::MockSymbol(const std::string& symbol, double base,
                                       std::chrono::system_clock::time_point now) {
  double last = base + RandFloat();
  double bid = last - 0.03;
  double ask = last + 0.03;
  double spread = ask - bid;
  double roi = (RandFloat() * 0.8 + 0.2) * 100.0;
  int maker = static_cast<int>(RandFloat() * 3);
  int taker = static_cast<int>(RandFloat() * 2);
  int volume = 80 + static_cast<int>(RandFloat() * 50);

  SymbolSnapshot sym;
  sym.symbol = symbol;
  sym.last = last;
  sym.bid = bid;
  sym.ask = ask;
  sym.spread = spread;
  sym.roi = roi;
  sym.maker_count = maker;
  sym.taker_count = taker;
  sym.volume = volume;
  sym.candle = MockCandle(last, base, now);
  sym.multiplier = 100.0;
  sym.option_chains = MockOptionChains(last, now);

  return sym;
}

Position MockProvider::MockPosition(const std::string& name, int qty,
                                   std::chrono::system_clock::time_point now) {
  Position pos;
  pos.name = name;
  pos.quantity = qty;
  pos.roi = (RandFloat() * 0.6 + 0.2) * 100.0;
  pos.maker_count = static_cast<int>(RandFloat() * 3);
  pos.taker_count = static_cast<int>(RandFloat() * 2);
  pos.rebate_estimate = RandFloat() * 2.0;
  pos.vega = RandFloat() * 0.5;
  pos.theta = (RandFloat() * 0.2 - 0.1);
  pos.fair_diff = (RandFloat() * 0.2 - 0.1) * 5.0;
  pos.candle = MockCandle(160.0 + RandFloat() * 5.0, 160.0, now);
  return pos;
}

Candle MockProvider::MockCandle(double current, double base,
                               std::chrono::system_clock::time_point now) {
  double high = std::max(current + RandFloat() * 0.5, current);
  double low = std::min(current - RandFloat() * 0.5, current);
  double open = base + RandFloat() * 0.5;
  double volume = 50.0 + RandFloat() * 20.0;

  Candle candle;
  candle.open = open;
  candle.high = high;
  candle.low = low;
  candle.close = current;
  candle.volume = volume;
  candle.entry = base;
  candle.updated = now;
  return candle;
}

std::vector<OptionSeries> MockProvider::MockOptionChains(double last,
                                                        std::chrono::system_clock::time_point now) {
  double rounded = std::round(last / 5.0) * 5.0;
  if (rounded <= 0) rounded = last;

  std::vector<OptionStrike> strikes;
  for (int i = -5; i <= 5; ++i) {
    double strike = rounded + i * 5.0;
    if (strike <= 0) continue;

    double intrinsic_call = std::max(last - strike, 0.0);
    double intrinsic_put = std::max(strike - last, 0.0);
    double call_mid = intrinsic_call + RandFloat() * 1.5 + 0.2;
    double put_mid = intrinsic_put + RandFloat() * 1.5 + 0.2;
    double call_spread = RandFloat() * 0.15 + 0.05;
    double put_spread = RandFloat() * 0.15 + 0.05;

    OptionStrike opt;
    opt.strike = strike;
    opt.call_bid = std::max(0.0, call_mid - call_spread / 2.0);
    opt.call_ask = call_mid + call_spread / 2.0;
    opt.put_bid = std::max(0.0, put_mid - put_spread / 2.0);
    opt.put_ask = put_mid + put_spread / 2.0;
    strikes.push_back(opt);
  }

  auto expiry = now + std::chrono::hours(30 * 24);
  auto time_t = std::chrono::system_clock::to_time_t(expiry);
  std::tm tm_buf;
  localtime_r(&time_t, &tm_buf);
  std::ostringstream oss;
  oss << std::setfill('0') << (1900 + tm_buf.tm_year) << "-"
      << std::setw(2) << (tm_buf.tm_mon + 1) << "-"
      << std::setw(2) << tm_buf.tm_mday;

  OptionSeries series;
  series.expiration = oss.str();
  series.strikes = strikes;

  return {series};
}

std::vector<HistoryEntry> MockProvider::MockHistory(std::chrono::system_clock::time_point now) {
  std::vector<HistoryEntry> records;
  std::vector<int> expiries = {7, 14, 28, 56, 91, 182};

  for (size_t i = 0; i < expiries.size(); ++i) {
    auto date = now - std::chrono::hours((i + 1) * 3 * 24);
    double width = (i % 2 == 0) ? 5.0 : 10.0;
    double net_debit = 100.0 + i * 12.5 + RandFloat() * 2.0;
    double apr = 4.5 + i * 0.35 + RandFloat() * 0.25;
    double benchmark = 4.8 + i * 0.1;

    auto expiry_date = now + std::chrono::hours(expiries[i] * 24);
    auto time_t = std::chrono::system_clock::to_time_t(expiry_date);
    std::tm tm_buf;
    localtime_r(&time_t, &tm_buf);
    std::ostringstream oss;
    oss << std::setfill('0') << (1900 + tm_buf.tm_year) << "-"
        << std::setw(2) << (tm_buf.tm_mon + 1) << "-"
        << std::setw(2) << tm_buf.tm_mday;

    HistoryEntry entry;
    entry.date = date;
    entry.symbol = "SPX";
    entry.expiration = oss.str();
    entry.width = width;
    entry.net_debit = net_debit;
    entry.apr = apr;
    entry.benchmark = "BIL";
    entry.benchmark_rate = benchmark;
    entry.notes = "Synthetic funding snapshot";
    entry.days_to_expiry = static_cast<double>(expiries[i]);
    // Default to European-style for SPX
    entry.option_style = "European";

    // Calculate buy vs sell disparity (simulate intraday differences)
    // Buy: using ASK prices, Sell: using BID prices
    double spread_width = width;
    double avg_bid_ask = 0.10 + RandFloat() * 0.30;  // 0.10 - 0.40 spread
    double buy_cost = net_debit + avg_bid_ask * 2.0;  // Buying costs more (ASK side)
    double sell_credit = net_debit - avg_bid_ask * 2.0;  // Selling receives less (BID side)
    entry.buy_profit = spread_width - buy_cost;
    entry.sell_profit = sell_credit - spread_width;
    entry.buy_sell_disparity = entry.buy_profit - entry.sell_profit;

    // Put-call parity violation (bps) - simulate small violations
    entry.put_call_parity_violation = (RandFloat() - 0.5) * 100.0;  // -50 to +50 bps

    records.push_back(entry);
  }

  return records;
}

std::vector<YieldCurvePoint> MockProvider::MockYieldCurve(std::chrono::system_clock::time_point now) {
  std::vector<YieldCurvePoint> points;
  std::vector<std::pair<std::string, int>> tenors = {
    {"12D", 12}, {"1M", 30}, {"2M", 60}, {"3M", 90},
    {"6M", 180}, {"9M", 270}, {"1Y", 360}, {"18M", 540}
  };

  for (size_t i = 0; i < tenors.size(); ++i) {
    double base_apr = 5.0 + i * 0.1;
    double apr = base_apr + (RandFloat() - 0.5) * 0.4;
    double benchmark = 4.8 + i * 0.08;
    double net_debit = 90.0 + i * 15.0 + RandFloat() * 3.0;

    auto expiry_date = now + std::chrono::hours(tenors[i].second * 24);
    auto time_t = std::chrono::system_clock::to_time_t(expiry_date);
    std::tm tm_buf;
    localtime_r(&time_t, &tm_buf);
    std::ostringstream oss;
    oss << std::setfill('0') << (1900 + tm_buf.tm_year) << "-"
        << std::setw(2) << (tm_buf.tm_mon + 1) << "-"
        << std::setw(2) << tm_buf.tm_mday;

    YieldCurvePoint point;
    point.label = tenors[i].first;
    point.expiration = oss.str();
    point.dte = static_cast<double>(tenors[i].second);
    point.net_debit = net_debit;
    point.apr = apr;
    point.benchmark = benchmark;
    point.apr_spread = apr - benchmark;
    // SPX is European-style
    point.option_style = "European";

    // Calculate buy vs sell disparity for yield curve points
    double spread_width = 5.0;  // Assume 5 point width for yield curve
    double avg_bid_ask = 0.15 + RandFloat() * 0.25;  // 0.15 - 0.40 spread
    double buy_cost = net_debit + avg_bid_ask * 2.0;
    double sell_credit = net_debit - avg_bid_ask * 2.0;
    double theoretical = spread_width;
    point.buy_profit = theoretical - buy_cost;
    point.sell_profit = sell_credit - theoretical;
    point.buy_sell_disparity = point.buy_profit - point.sell_profit;

    // Put-call parity violation
    point.put_call_parity_violation = (RandFloat() - 0.5) * 80.0;  // -40 to +40 bps

    points.push_back(point);
  }

  return points;
}

std::vector<FAQEntry> MockProvider::MockFAQs() {
  return {
    {
      "What is a box spread?",
      "A four-leg options strategy combining a bull call spread and bear put spread with matching strikes to synthetically borrow or lend cash."
    },
    {
      "How is the APR calculated?",
      "APR is annualized from the net profit percentage using 365-day basis: APR = profit_pct * (365 / days_to_expiry)."
    },
    {
      "Why compare against T-bill benchmarks?",
      "Treasury bills provide the prevailing risk-free funding rate. Comparing APR to a nearby tenor highlights funding edge or drag."
    }
  };
}

// ============================================================================
// RestProvider Implementation
// ============================================================================

RestProvider::RestProvider(const std::string& endpoint, std::chrono::milliseconds interval)
  : endpoint_(endpoint), interval_(interval) {
}

RestProvider::~RestProvider() {
  Stop();
}

void RestProvider::Start() {
  if (running_.exchange(true)) {
    return;
  }

  worker_ = std::thread(&RestProvider::PollLoop, this);
  spdlog::info("RestProvider started with endpoint: {}", endpoint_);
}

void RestProvider::Stop() {
  if (!running_.exchange(false)) {
    return;
  }

  if (worker_.joinable()) {
    worker_.join();
  }
  spdlog::info("RestProvider stopped");
}

Snapshot RestProvider::GetSnapshot() {
  // Non-blocking: just return latest snapshot atomically
  std::lock_guard<std::mutex> lock(mutex_);
  return latest_snapshot_;
}

bool RestProvider::IsRunning() const {
  return running_.load();
}

void RestProvider::PollLoop() {
  // Try to fetch initial snapshot
  try {
    auto snap = Fetch();
    if (snap.generated_at.time_since_epoch().count() > 0) {
      std::lock_guard<std::mutex> lock(mutex_);
      latest_snapshot_ = snap;
    }
  } catch (...) {
    spdlog::warn("Initial REST fetch failed");
  }

  while (running_.load()) {
    try {
      auto snap = Fetch();
      {
        std::lock_guard<std::mutex> lock(mutex_);
        latest_snapshot_ = snap;
      }
    } catch (const std::exception& e) {
      spdlog::error("REST fetch error: {}", e.what());
    }
    std::this_thread::sleep_for(interval_);
  }
}

Snapshot RestProvider::Fetch() {
  // TODO: Implement HTTP client using curl or similar
  // For now, return empty snapshot
  spdlog::warn("REST provider not fully implemented yet");
  return Snapshot{};
}

// ============================================================================
// IBKRRestProvider Implementation
// ============================================================================

IBKRRestProvider::IBKRRestProvider(const std::string& base_url,
                                   const std::string& account_id,
                                   bool verify_ssl,
                                   std::chrono::milliseconds interval)
  : base_url_(base_url)
  , account_id_(account_id)
  , verify_ssl_(verify_ssl)
  , interval_(interval) {
  // Remove trailing slash from base_url
  if (!base_url_.empty() && base_url_.back() == '/') {
    base_url_.pop_back();
  }
}

IBKRRestProvider::~IBKRRestProvider() {
  Stop();
}

void IBKRRestProvider::Start() {
  if (running_.exchange(true)) {
    return;  // Already running
  }

  worker_ = std::thread(&IBKRRestProvider::PollLoop, this);
  spdlog::info("IBKRRestProvider started (base_url: {})", base_url_);
}

void IBKRRestProvider::Stop() {
  if (!running_.exchange(false)) {
    return;  // Already stopped
  }

  if (worker_.joinable()) {
    worker_.join();
  }
  spdlog::info("IBKRRestProvider stopped");
}

Snapshot IBKRRestProvider::GetSnapshot() {
  // Non-blocking: just return latest snapshot atomically
  std::lock_guard<std::mutex> lock(mutex_);
  return latest_snapshot_;
}

bool IBKRRestProvider::IsRunning() const {
  return running_.load();
}

void IBKRRestProvider::PollLoop() {
  // Try to fetch initial snapshot
  try {
    auto snap = FetchFromIBKR();
    if (snap.generated_at.time_since_epoch().count() > 0) {
      std::lock_guard<std::mutex> lock(mutex_);
      latest_snapshot_ = snap;
    }
  } catch (...) {
    spdlog::warn("Initial IBKR REST fetch failed");
  }

  while (running_.load()) {
    try {
      auto snap = FetchFromIBKR();
      {
        std::lock_guard<std::mutex> lock(mutex_);
        latest_snapshot_ = snap;
      }
    } catch (const std::exception& e) {
      spdlog::error("IBKR REST fetch error: {}", e.what());
    }
    std::this_thread::sleep_for(interval_);
  }
}

Snapshot IBKRRestProvider::FetchFromIBKR() {
  // TODO: Implement HTTP client using curl or similar
  // IBKR Client Portal API endpoints:
  // - GET /iserver/accounts - list accounts
  // - GET /iserver/account/{id}/summary - account summary
  // - GET /iserver/account/{id}/positions - positions
  // - GET /sso/validate - validate session
  // - POST /iserver/reauthenticate - reauthenticate

  // Ensure session is valid
  if (!EnsureSession()) {
    spdlog::warn("IBKR session validation failed");
    return Snapshot{};
  }

  // Get account ID if not specified
  if (active_account_id_.empty()) {
    auto accounts = GetAccounts();
    if (accounts.empty()) {
      spdlog::warn("No IBKR accounts available");
      return Snapshot{};
    }
    active_account_id_ = account_id_.empty() ? accounts[0] : account_id_;
  }

  // TODO: Fetch account summary and positions
  // For now, return empty snapshot
  spdlog::warn("IBKR REST provider not fully implemented yet - HTTP client needed");

  Snapshot snap;
  snap.generated_at = std::chrono::system_clock::now();
  snap.mode = "DRY-RUN";
  snap.strategy = "STOPPED";
  snap.account_id = active_account_id_;
  return snap;
}

bool IBKRRestProvider::EnsureSession() {
  // TODO: Implement session validation
  // GET {base_url_}/sso/validate
  // If 200 OK, session is valid
  // If not, POST {base_url_}/iserver/reauthenticate
  return true;  // Stub
}

std::vector<std::string> IBKRRestProvider::GetAccounts() {
  // TODO: Implement account listing
  // GET {base_url_}/iserver/accounts
  // Returns JSON: {"accounts": ["DU123456", ...]}
  return {};  // Stub
}

// ============================================================================
// LiveVolProvider Implementation
// ============================================================================

LiveVolProvider::LiveVolProvider(const std::string& base_url,
                                const std::string& client_id,
                                const std::string& client_secret,
                                bool use_real_time,
                                std::chrono::milliseconds interval)
  : base_url_(base_url)
  , client_id_(client_id)
  , client_secret_(client_secret)
  , use_real_time_(use_real_time)
  , interval_(interval) {
  // Remove trailing slash from base_url
  if (!base_url_.empty() && base_url_.back() == '/') {
    base_url_.pop_back();
  }
}

LiveVolProvider::~LiveVolProvider() {
  Stop();
}

void LiveVolProvider::Start() {
  if (running_.exchange(true)) {
    return;  // Already running
  }

  worker_ = std::thread(&LiveVolProvider::PollLoop, this);
  spdlog::info("LiveVolProvider started (base_url: {})", base_url_);
}

void LiveVolProvider::Stop() {
  if (!running_.exchange(false)) {
    return;  // Already stopped
  }

  if (worker_.joinable()) {
    worker_.join();
  }
  spdlog::info("LiveVolProvider stopped");
}

Snapshot LiveVolProvider::GetSnapshot() {
  // Non-blocking: just return latest snapshot atomically
  std::lock_guard<std::mutex> lock(mutex_);
  return latest_snapshot_;
}

bool LiveVolProvider::IsRunning() const {
  return running_.load();
}

void LiveVolProvider::PollLoop() {
  // Try to fetch initial snapshot
  try {
    auto snap = FetchFromLiveVol();
    if (snap.generated_at.time_since_epoch().count() > 0) {
      std::lock_guard<std::mutex> lock(mutex_);
      latest_snapshot_ = snap;
    }
  } catch (...) {
    spdlog::warn("Initial LiveVol fetch failed");
  }

  while (running_.load()) {
    try {
      auto snap = FetchFromLiveVol();
      {
        std::lock_guard<std::mutex> lock(mutex_);
        latest_snapshot_ = snap;
      }
    } catch (const std::exception& e) {
      spdlog::error("LiveVol fetch error: {}", e.what());
    }
    std::this_thread::sleep_for(interval_);
  }
}

Snapshot LiveVolProvider::FetchFromLiveVol() {
  // TODO: Implement HTTP client using curl or similar
  // LiveVol API endpoints (see https://api.livevol.com/v1/docs/):
  // - OAuth 2.0: POST /oauth/token - Get access token
  // - Market Data: GET /allaccess/time-and-sales/quotes - Real-time quotes
  // - Option Strategy Scans: GET /allaccess/strategy-scans - Strategy results
  // - Historical Data: GET /allaccess/historical - Historical time-series

  // Ensure access token is valid
  if (!EnsureAccessToken()) {
    spdlog::warn("LiveVol access token validation failed");
    return Snapshot{};
  }

  // TODO: Fetch market data and options data
  // For now, return empty snapshot
  spdlog::warn("LiveVol provider not fully implemented yet - HTTP client needed");

  Snapshot snap;
  snap.generated_at = std::chrono::system_clock::now();
  snap.mode = "DRY-RUN";
  snap.strategy = "STOPPED";
  return snap;
}

bool LiveVolProvider::EnsureAccessToken() {
  // Check if token exists and is not expired
  if (!access_token_.empty() &&
      std::chrono::system_clock::now() < token_expiry_) {
    return true;  // Token is still valid
  }

  // Need to get new token
  access_token_ = GetAccessToken();
  return !access_token_.empty();
}

std::string LiveVolProvider::GetAccessToken() {
  // TODO: Implement OAuth 2.0 token request
  // POST {base_url_}/oauth/token
  // Body: grant_type=client_credentials&client_id={client_id}&client_secret={client_secret}
  // Returns: {"access_token": "...", "expires_in": 3600, ...}
  // Set token_expiry_ = now + expires_in

  if (client_id_.empty() || client_secret_.empty()) {
    spdlog::error("LiveVol credentials not configured");
    return "";
  }

  return "";  // Stub
}

} // namespace tui
