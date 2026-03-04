// TWS_INTEGRATION_TEMPLATE.cpp
// This is a template showing how to integrate the actual TWS API
// Replace the stub implementation in src/tws_client.cpp with this approach

/*
PREREQUISITES:
1. Download TWS API from https://interactivebrokers.github.io/
2. Extract to native/third_party/tws-api/
3. Include these headers:
*/

#include "tws_client.h"
#include <spdlog/spdlog.h>

// TWS API headers (after installation)
#include "Contract.h"
#include "EClientSocket.h"
#include "EReader.h"
#include "EReaderOSSignal.h"
#include "EWrapper.h"
#include "Order.h"

#include <atomic>
#include <condition_variable>
#include <mutex>
#include <thread>

namespace tws {

// ============================================================================
// TWSClient::Impl - Real Implementation with TWS API
// ============================================================================

class TWSClient::Impl : public EWrapper {
public:
  explicit Impl(const config::TWSConfig &config)
      : config_(config), signal_(2000) // 2 second timeout
        ,
        client_(this, &signal_), next_order_id_(0), connected_(false) {}

  ~Impl() { disconnect(); }

  // ========================================================================
  // Connection Management
  // ========================================================================

  bool connect() {
    spdlog::info("Connecting to TWS at {}:{}...", config_.host, config_.port);

    // Connect to TWS
    bool success =
        client_.eConnect(config_.host.c_str(), config_.port, config_.client_id);

    if (!success) {
      spdlog::error("Failed to connect to TWS");
      return false;
    }

    // Wait for connection acknowledgment
    if (!wait_for_connection(config_.connection_timeout_ms)) {
      spdlog::error("Connection timeout");
      client_.eDisconnect();
      return false;
    }

    // Start message reader thread
    start_reader_thread();

    spdlog::info("✓ Connected to TWS");
    return true;
  }

  void disconnect() {
    if (connected_) {
      spdlog::info("Disconnecting from TWS...");

      if (reader_thread_ && reader_thread_->joinable()) {
        connected_ = false;
        reader_thread_->join();
      }

      client_.eDisconnect();
      spdlog::info("✓ Disconnected");
    }
  }

  bool is_connected() const { return connected_ && client_.isConnected(); }

  // ========================================================================
  // EWrapper Callbacks - Connection
  // ========================================================================

  void connectAck() override {
    spdlog::info("Connection acknowledged by TWS");

    // Request next valid order ID
    client_.reqIds(-1);
  }

  void connectionClosed() override {
    spdlog::warn("Connection closed by TWS");
    connected_ = false;

    // Auto-reconnect if enabled
    if (config_.auto_reconnect) {
      std::this_thread::sleep_for(
          std::chrono::milliseconds(config_.reconnect_delay_ms));
      connect();
    }
  }

  void nextValidId(OrderId orderId) override {
    spdlog::info("Received nextValidId: {}", orderId);
    next_order_id_ = orderId;

    std::lock_guard<std::mutex> lock(connection_mutex_);
    connected_ = true;
    connection_cv_.notify_all();
  }

  // ========================================================================
  // EWrapper Callbacks - Market Data
  // ========================================================================

  void tickPrice(TickerId tickerId, TickType field, double price,
                 const TickAttrib &attribs) override {
    spdlog::trace("tickPrice: id={}, field={}, price={}", tickerId, field,
                  price);

    std::lock_guard<std::mutex> lock(data_mutex_);

    auto &market_data = market_data_[tickerId];

    switch (field) {
    case BID:
      market_data.bid = price;
      break;
    case ASK:
      market_data.ask = price;
      break;
    case LAST:
      market_data.last = price;
      break;
    default:
      break;
    }

    market_data.timestamp = std::chrono::system_clock::now();

    // Notify callback if registered
    if (market_data_callbacks_.count(tickerId)) {
      market_data_callbacks_[tickerId](market_data);
    }
  }

  void tickSize(TickerId tickerId, TickType field, Decimal size) override {
    spdlog::trace("tickSize: id={}, field={}, size={}", tickerId, field, size);

    std::lock_guard<std::mutex> lock(data_mutex_);

    auto &market_data = market_data_[tickerId];

    switch (field) {
    case BID_SIZE:
      market_data.bid_size = static_cast<int>(size);
      break;
    case ASK_SIZE:
      market_data.ask_size = static_cast<int>(size);
      break;
    case LAST_SIZE:
      market_data.last_size = static_cast<int>(size);
      break;
    case VOLUME:
      market_data.volume = static_cast<double>(size);
      break;
    default:
      break;
    }
  }

  void tickOptionComputation(TickerId tickerId, TickType tickType,
                             int tickAttrib, double impliedVol, double delta,
                             double optPrice, double pvDividend, double gamma,
                             double vega, double theta,
                             double undPrice) override {
    spdlog::trace("tickOptionComputation: id={}, IV={}, delta={}", tickerId,
                  impliedVol, delta);

    std::lock_guard<std::mutex> lock(data_mutex_);

    auto &market_data = market_data_[tickerId];

    if (impliedVol >= 0) {
      market_data.implied_volatility = impliedVol;
    }
    if (delta != DBL_MAX) {
      market_data.delta = delta;
    }
    if (gamma != DBL_MAX) {
      market_data.gamma = gamma;
    }
    if (vega != DBL_MAX) {
      market_data.vega = vega;
    }
    if (theta != DBL_MAX) {
      market_data.theta = theta;
    }
  }

  // ========================================================================
  // EWrapper Callbacks - Orders
  // ========================================================================

  void orderStatus(OrderId orderId, const std::string &status, Decimal filled,
                   Decimal remaining, double avgFillPrice, int permId,
                   int parentId, double lastFillPrice, int clientId,
                   const std::string &whyHeld, double mktCapPrice) override {
    spdlog::info("Order #{} status: {}, filled={}, remaining={}", orderId,
                 status, filled, remaining);

    std::lock_guard<std::mutex> lock(order_mutex_);

    if (orders_.count(orderId)) {
      auto &order = orders_[orderId];

      // Update status
      if (status == "PreSubmitted" || status == "Submitted") {
        order.status = types::OrderStatus::Submitted;
      } else if (status == "Filled") {
        order.status = types::OrderStatus::Filled;
      } else if (status == "Cancelled") {
        order.status = types::OrderStatus::Cancelled;
      } else if (status == "Inactive" || status == "ApiCancelled") {
        order.status = types::OrderStatus::Rejected;
      }

      order.filled_quantity = static_cast<int>(filled);
      order.avg_fill_price = avgFillPrice;
      order.last_update = std::chrono::system_clock::now();

      // Notify callback
      if (order_status_callback_) {
        order_status_callback_(order);
      }
    }
  }

  void openOrder(OrderId orderId, const Contract &contract, const Order &order,
                 const OrderState &orderState) override {
    spdlog::debug("Open order: #{}, {}, {}", orderId, contract.symbol,
                  order.action);
    // Handle open order details
  }

  void execDetails(int reqId, const Contract &contract,
                   const Execution &execution) override {
    spdlog::info("Execution: order={}, shares={}, price={}", execution.orderId,
                 execution.shares, execution.price);
    // Handle execution details
  }

  // ========================================================================
  // EWrapper Callbacks - Account & Positions
  // ========================================================================

  void position(const std::string &account, const Contract &contract,
                Decimal position, double avgCost) override {
    spdlog::debug("Position: {} {} @ {}", position, contract.symbol, avgCost);
    // Handle position updates
  }

  void updateAccountValue(const std::string &key, const std::string &val,
                          const std::string &currency,
                          const std::string &accountName) override {
    spdlog::trace("Account update: {}={}", key, val);

    std::lock_guard<std::mutex> lock(account_mutex_);

    // Update account info based on key
    if (key == "NetLiquidation") {
      account_info_.net_liquidation = std::stod(val);
    } else if (key == "TotalCashBalance") {
      account_info_.cash_balance = std::stod(val);
    } else if (key == "BuyingPower") {
      account_info_.buying_power = std::stod(val);
    }
    // ... handle other account fields
  }

  void updatePortfolio(const Contract &contract, Decimal position,
                       double marketPrice, double marketValue,
                       double averageCost, double unrealizedPNL,
                       double realizedPNL,
                       const std::string &accountName) override {
    spdlog::debug("Portfolio update: {} position={}, PnL={}", contract.symbol,
                  position, unrealizedPNL);
    // Handle portfolio updates
  }

  // ========================================================================
  // EWrapper Callbacks - Error Handling
  // ========================================================================

  void error(int id, int errorCode, const std::string &errorString,
             const std::string &advancedOrderRejectJson) override {
    if (errorCode >= 2100 && errorCode < 3000) {
      // Informational messages
      spdlog::info("TWS message {}: {}", errorCode, errorString);
    } else if (errorCode >= 1100 && errorCode < 2000) {
      // System messages
      spdlog::warn("TWS system {}: {}", errorCode, errorString);
    } else {
      // Errors
      spdlog::error("TWS error {} (id={}): {}", errorCode, id, errorString);

      if (error_callback_) {
        error_callback_(errorCode, errorString);
      }
    }
  }

  // ========================================================================
  // Market Data Operations (Public Interface)
  // ========================================================================

  int request_market_data(const types::OptionContract &contract,
                          MarketDataCallback callback) {
    int request_id = next_request_id_++;

    // Convert to TWS Contract
    Contract tws_contract = convert_to_tws_contract(contract);

    // Register callback
    {
      std::lock_guard<std::mutex> lock(data_mutex_);
      market_data_callbacks_[request_id] = callback;
    }

    // Request market data
    client_.reqMktData(request_id,        // Request ID
                       tws_contract,      // Contract
                       "",                // Generic tick list
                       false,             // Snapshot
                       false,             // Regulatory snapshot
                       TagValueListSPtr() // Options
    );

    spdlog::debug("Requested market data for {} (id={})", contract.to_string(),
                  request_id);

    return request_id;
  }

  void cancel_market_data(int request_id) {
    client_.cancelMktData(request_id);

    std::lock_guard<std::mutex> lock(data_mutex_);
    market_data_callbacks_.erase(request_id);
  }

  // ========================================================================
  // Order Operations (Public Interface)
  // ========================================================================

  int place_order(const types::OptionContract &contract,
                  types::OrderAction action, int quantity, double limit_price,
                  types::TimeInForce tif) {
    int order_id = next_order_id_++;

    // Convert to TWS types
    Contract tws_contract = convert_to_tws_contract(contract);
    Order tws_order = create_tws_order(action, quantity, limit_price, tif);

    // Place order
    client_.placeOrder(order_id, tws_contract, tws_order);

    spdlog::info("Placed order #{}: {} {} {} @ {}", order_id,
                 types::order_action_to_string(action), quantity,
                 contract.to_string(),
                 limit_price > 0 ? std::to_string(limit_price) : "MKT");

    // Store order
    {
      std::lock_guard<std::mutex> lock(order_mutex_);
      types::Order our_order;
      our_order.order_id = order_id;
      our_order.contract = contract;
      our_order.action = action;
      our_order.quantity = quantity;
      our_order.limit_price = limit_price;
      our_order.tif = tif;
      our_order.status = types::OrderStatus::Submitted;
      our_order.submitted_time = std::chrono::system_clock::now();
      orders_[order_id] = our_order;
    }

    return order_id;
  }

  void cancel_order(int order_id) {
    client_.cancelOrder(order_id, "");
    spdlog::info("Cancelled order #{}", order_id);
  }

  // ========================================================================
  // Helper Methods
  // ========================================================================

private:
  void start_reader_thread() {
    auto reader = std::make_unique<EReader>(&client_, &signal_);
    reader->start();

    reader_thread_ =
        std::make_unique<std::thread>([this, r = std::move(reader)]() {
          while (connected_) {
            signal_.waitForSignal();
            r->processMsgs();
          }
        });
  }

  bool wait_for_connection(int timeout_ms) {
    std::unique_lock<std::mutex> lock(connection_mutex_);
    return connection_cv_.wait_for(lock, std::chrono::milliseconds(timeout_ms),
                                   [this] { return connected_.load(); });
  }

  Contract convert_to_tws_contract(const types::OptionContract &contract) {
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

  Order create_tws_order(types::OrderAction action, int quantity,
                         double limit_price, types::TimeInForce tif) {
    Order o;
    o.action = (action == types::OrderAction::Buy) ? "BUY" : "SELL";
    o.totalQuantity = quantity;
    o.orderType = (limit_price > 0) ? "LMT" : "MKT";

    if (limit_price > 0) {
      o.lmtPrice = limit_price;
    }

    // Set time in force
    switch (tif) {
    case types::TimeInForce::Day:
      o.tif = "DAY";
      break;
    case types::TimeInForce::GTC:
      o.tif = "GTC";
      break;
    case types::TimeInForce::IOC:
      o.tif = "IOC";
      break;
    case types::TimeInForce::FOK:
      o.tif = "FOK";
      break;
    }

    return o;
  }

  // ========================================================================
  // Member Variables
  // ========================================================================

  config::TWSConfig config_;
  EReaderOSSignal signal_;
  EClientSocket client_;

  std::atomic<bool> connected_;
  std::atomic<int> next_order_id_;
  std::atomic<int> next_request_id_{1000};

  std::unique_ptr<std::thread> reader_thread_;

  // Connection synchronization
  std::mutex connection_mutex_;
  std::condition_variable connection_cv_;

  // Market data
  std::mutex data_mutex_;
  std::map<int, types::MarketData> market_data_;
  std::map<int, MarketDataCallback> market_data_callbacks_;

  // Orders
  std::mutex order_mutex_;
  std::map<int, types::Order> orders_;
  OrderStatusCallback order_status_callback_;

  // Account
  std::mutex account_mutex_;
  types::AccountInfo account_info_;

  // Callbacks
  ErrorCallback error_callback_;

  // ========================================================================
  // Remaining EWrapper callbacks (stubs for now - implement as needed)
  // ========================================================================

  void tickString(TickerId tickerId, TickType tickType,
                  const std::string &value) override {}
  void tickEFP(TickerId tickerId, TickType tickType, double basisPoints,
               const std::string &formattedBasisPoints, double totalDividends,
               int holdDays, const std::string &futureLastTradeDate,
               double dividendImpact,
               double dividendsToLastTradeDate) override {}
  void tickGeneric(TickerId tickerId, TickType tickType,
                   double value) override {}
  void tickSnapshotEnd(int reqId) override {}
  void marketDataType(TickerId reqId, int marketDataType) override {}
  void realtimeBar(TickerId reqId, long time, double open, double high,
                   double low, double close, Decimal volume, Decimal wap,
                   int count) override {}
  void historicalData(TickerId reqId, const Bar &bar) override {}
  void historicalDataEnd(int reqId, const std::string &startDateStr,
                         const std::string &endDateStr) override {}
  void scannerParameters(const std::string &xml) override {}
  void scannerData(int reqId, int rank, const ContractDetails &contractDetails,
                   const std::string &distance, const std::string &benchmark,
                   const std::string &projection,
                   const std::string &legsStr) override {}
  void scannerDataEnd(int reqId) override {}
  void receiveFA(faDataType pFaDataType, const std::string &cxml) override {}
  void bondContractDetails(int reqId,
                           const ContractDetails &contractDetails) override {}
  void contractDetails(int reqId,
                       const ContractDetails &contractDetails) override {}
  void contractDetailsEnd(int reqId) override {}
  void execDetailsEnd(int reqId) override {}
  void updateAccountTime(const std::string &timeStamp) override {}
  void accountDownloadEnd(const std::string &accountName) override {}
  void positionEnd() override {}
  void accountSummary(int reqId, const std::string &account,
                      const std::string &tag, const std::string &value,
                      const std::string &currency) override {}
  void accountSummaryEnd(int reqId) override {}
  void verifyMessageAPI(const std::string &apiData) override {}
  void verifyCompleted(bool isSuccessful,
                       const std::string &errorText) override {}
  void verifyAndAuthMessageAPI(const std::string &apiData,
                               const std::string &xyzChallange) override {}
  void verifyAndAuthCompleted(bool isSuccessful,
                              const std::string &errorText) override {}
  void displayGroupList(int reqId, const std::string &groups) override {}
  void displayGroupUpdated(int reqId,
                           const std::string &contractInfo) override {}
  void positionMulti(int reqId, const std::string &account,
                     const std::string &modelCode, const Contract &contract,
                     Decimal pos, double avgCost) override {}
  void positionMultiEnd(int reqId) override {}
  void accountUpdateMulti(int reqId, const std::string &account,
                          const std::string &modelCode, const std::string &key,
                          const std::string &value,
                          const std::string &currency) override {}
  void accountUpdateMultiEnd(int reqId) override {}
  void securityDefinitionOptionalParameter(
      int reqId, const std::string &exchange, int underlyingConId,
      const std::string &tradingClass, const std::string &multiplier,
      const std::set<std::string> &expirations,
      const std::set<double> &strikes) override {}
  void securityDefinitionOptionalParameterEnd(int reqId) override {}
  void softDollarTiers(int reqId,
                       const std::vector<SoftDollarTier> &tiers) override {}
  void familyCodes(const std::vector<FamilyCode> &familyCodes) override {}
  void symbolSamples(
      int reqId,
      const std::vector<ContractDescription> &contractDescriptions) override {}
  void mktDepthExchanges(const std::vector<DepthMktDataDescription>
                             &depthMktDataDescriptions) override {}
  void tickNews(int tickerId, time_t timeStamp, const std::string &providerCode,
                const std::string &articleId, const std::string &headline,
                const std::string &extraData) override {}
  void smartComponents(int reqId, const SmartComponentsMap &theMap) override {}
  void tickReqParams(int tickerId, double minTick,
                     const std::string &bboExchange,
                     int snapshotPermissions) override {}
  void newsProviders(const std::vector<NewsProvider> &newsProviders) override {}
  void newsArticle(int requestId, int articleType,
                   const std::string &articleText) override {}
  void historicalNews(int requestId, const std::string &time,
                      const std::string &providerCode,
                      const std::string &articleId,
                      const std::string &headline) override {}
  void historicalNewsEnd(int requestId, bool hasMore) override {}
  void headTimestamp(int reqId, const std::string &headTimestamp) override {}
  void histogramData(int reqId, const HistogramDataVector &data) override {}
  void historicalDataUpdate(TickerId reqId, const Bar &bar) override {}
  void rerouteMktDataReq(int reqId, int conid,
                         const std::string &exchange) override {}
  void rerouteMktDepthReq(int reqId, int conid,
                          const std::string &exchange) override {}
  void marketRule(int marketRuleId,
                  const std::vector<PriceIncrement> &priceIncrements) override {
  }
  void pnl(int reqId, double dailyPnL, double unrealizedPnL,
           double realizedPnL) override {}
  void pnlSingle(int reqId, Decimal pos, double dailyPnL, double unrealizedPnL,
                 double realizedPnL, double value) override {}
  void historicalTicks(int reqId, const std::vector<HistoricalTick> &ticks,
                       bool done) override {}
  void historicalTicksBidAsk(int reqId,
                             const std::vector<HistoricalTickBidAsk> &ticks,
                             bool done) override {}
  void historicalTicksLast(int reqId,
                           const std::vector<HistoricalTickLast> &ticks,
                           bool done) override {}
  void tickByTickAllLast(int reqId, int tickType, time_t time, double price,
                         Decimal size, const TickAttribLast &tickAttribLast,
                         const std::string &exchange,
                         const std::string &specialConditions) override {}
  void tickByTickBidAsk(int reqId, time_t time, double bidPrice,
                        double askPrice, Decimal bidSize, Decimal askSize,
                        const TickAttribBidAsk &tickAttribBidAsk) override {}
  void tickByTickMidPoint(int reqId, time_t time, double midPoint) override {}
  void orderBound(long long orderId, int apiClientId, int apiOrderId) override {
  }
  void completedOrder(const Contract &contract, const Order &order,
                      const OrderState &orderState) override {}
  void completedOrdersEnd() override {}
  void replaceFAEnd(int reqId, const std::string &text) override {}
  void wshMetaData(int reqId, const std::string &dataJson) override {}
  void wshEventData(int reqId, const std::string &dataJson) override {}
  void
  historicalSchedule(int reqId, const std::string &startDateTime,
                     const std::string &endDateTime,
                     const std::string &timeZone,
                     const std::vector<HistoricalSession> &sessions) override {}
  void userInfo(int reqId, const std::string &whiteBrandingId) override {}
};

// ============================================================================
// TWSClient Public Interface (delegates to Impl)
// ============================================================================

TWSClient::TWSClient(const config::TWSConfig &config)
    : pimpl_(std::make_unique<Impl>(config)) {}

TWSClient::~TWSClient() = default;

bool TWSClient::connect() { return pimpl_->connect(); }

void TWSClient::disconnect() { pimpl_->disconnect(); }

bool TWSClient::is_connected() const { return pimpl_->is_connected(); }

// ... implement other public methods by delegating to pimpl_ ...

} // namespace tws

/*
NEXT STEPS:

1. Copy this template structure to src/tws_client.cpp
2. Adjust includes based on your TWS API installation
3. Implement remaining callbacks as needed
4. Test with paper trading account
5. Iterate and improve based on actual behavior

NOTES:

- This is a STARTING POINT - you'll need to adapt it
- Error handling needs to be more robust
- Thread safety is critical - review all mutex usage
- TWS API is complex - study the official examples
- Test thoroughly before live trading!
*/
