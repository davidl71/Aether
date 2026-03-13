// tws_client_impl.h - TWSClient::Impl class declaration (split from tws_client.cpp)
#pragma once

#include "tws_client.h"
#include "config_manager.h"
#include "rate_limiter.h"
#include "market_hours.h"
#include "margin_calculator.h"
#include "pcap_capture.h"
#include "types.h"
#include "cache_client.h"
#include "Contract.h"
#include "DefaultEWrapper.h"
#include "EClientSocket.h"
#include "EReaderOSSignal.h"
#include "Execution.h"
#include "Order.h"
#include "OrderState.h"
#include "EWrapper.h"

#ifdef ENABLE_NATS
#include "nats_client.h"
#endif

#include <atomic>
#include <chrono>
#include <condition_variable>
#include <map>
#include <memory>
#include <mutex>
#include <set>
#include <string>
#include <thread>
#include <unordered_map>
#include <vector>
#include <future>
#include <optional>

namespace tws {

using TickerId = int;
using OrderId = int;

class TWSClient::Impl : public DefaultEWrapper {
 public:
  explicit Impl(const config::TWSConfig& config);
  ~Impl();

  // Connection
  bool connect();
  void disconnect();
  bool is_connected() const;
  ConnectionState get_connection_state() const;
  void process_messages(int timeout_ms);

  // EWrapper callbacks (defined in tws_client_impl_ewrapper.cpp)
  void connectAck() override;
  void connectionClosed() override;
  void managedAccounts(const std::string& accountsList) override;
  void nextValidId(OrderId orderId) override;
  void tickPrice(TickerId tickerId, TickType field, double price,
                 const TickAttrib& attribs) override;
  void tickSize(TickerId tickerId, TickType field, Decimal size) override;
  void tickOptionComputation(TickerId tickerId, TickType tickType, int tickAttrib,
                             double impliedVol, double delta, double optPrice,
                             double pvDividend, double gamma, double vega,
                             double theta, double undPrice) override;
  void orderStatus(OrderId orderId, const std::string& status, Decimal filled,
                   Decimal remaining, double avgFillPrice, long long permId,
                   int parentId, double lastFillPrice, int clientId,
                   const std::string& whyHeld, double mktCapPrice) override;
  void openOrder(OrderId orderId, const Contract& contract, const Order& order,
                 const OrderState& orderState) override;
  void openOrderEnd() override;
  void execDetails(int reqId, const Contract& contract,
                   const Execution& execution) override;
  void execDetailsEnd(int reqId) override;
  void position(const std::string& account, const Contract& contract,
                Decimal position, double avgCost) override;
  void positionEnd() override;
  void updateAccountValue(const std::string& key, const std::string& val,
                          const std::string& currency,
                          const std::string& accountName) override;
  void updateAccountTime(const std::string& timeStamp) override;
  void accountDownloadEnd(const std::string& accountName) override;
  void updatePortfolio(const Contract& contract, Decimal position,
                       double marketPrice, double marketValue,
                       double averageCost, double unrealizedPNL,
                       double realizedPNL, const std::string& accountName) override;
  void currentTime(long long time) override;
  void error(int id, time_t errorTime, int errorCode,
             const std::string& errorString,
             const std::string& advancedOrderRejectJson) override;

  void tickString(TickerId tickerId, TickType tickType,
                  const std::string& value) override;
  void tickEFP(TickerId tickerId, TickType tickType, double basisPoints,
               const std::string& formattedBasisPoints, double totalDividends,
               int holdDays, const std::string& futureLastTradeDate,
               double dividendImpact, double dividendsToLastTradeDate) override;
  void tickGeneric(TickerId tickerId, TickType tickType, double value) override;
  void tickSnapshotEnd(int reqId) override;
  void marketDataType(TickerId reqId, int marketDataType) override;
  void realtimeBar(TickerId reqId, long long time, double open, double high,
                   double low, double close, Decimal volume, Decimal wap,
                   int count) override;
  void historicalData(TickerId reqId, const Bar& bar) override;
  void historicalDataEnd(int reqId, const std::string& startDateStr,
                        const std::string& endDateStr) override;
  void scannerParameters(const std::string& xml) override;
  void scannerData(int reqId, int rank, const ContractDetails& contractDetails,
                  const std::string& distance, const std::string& benchmark,
                  const std::string& projection, const std::string& legsStr) override;
  void scannerDataEnd(int reqId) override;
  void receiveFA(faDataType pFaDataType, const std::string& cxml) override;
  void bondContractDetails(int reqId,
                          const ContractDetails& contractDetails) override;
  void contractDetails(int reqId,
                       const ContractDetails& contractDetails) override;
  void contractDetailsEnd(int reqId) override;
  void accountSummary(int reqId, const std::string& account,
                      const std::string& tag, const std::string& value,
                      const std::string& currency) override;
  void accountSummaryEnd(int reqId) override;
  void verifyMessageAPI(const std::string& apiData) override;
  void verifyCompleted(bool isSuccessful, const std::string& errorText) override;
  void verifyAndAuthMessageAPI(const std::string& apiData,
                               const std::string& xyzChallange) override;
  void verifyAndAuthCompleted(bool isSuccessful,
                              const std::string& errorText) override;
  void displayGroupList(int reqId, const std::string& groups) override;
  void displayGroupUpdated(int reqId,
                          const std::string& contractInfo) override;
  void positionMulti(int reqId, const std::string& account,
                     const std::string& modelCode, const Contract& contract,
                     Decimal pos, double avgCost) override;
  void positionMultiEnd(int reqId) override;
  void accountUpdateMulti(int reqId, const std::string& account,
                          const std::string& modelCode, const std::string& key,
                          const std::string& value,
                          const std::string& currency) override;
  void accountUpdateMultiEnd(int reqId) override;
  void securityDefinitionOptionalParameter(
      int reqId, const std::string& exchange, int underlyingConId,
      const std::string& tradingClass, const std::string& multiplier,
      const std::set<std::string>& expirations,
      const std::set<double>& strikes) override;
  void securityDefinitionOptionalParameterEnd(int reqId) override;
  void softDollarTiers(int reqId,
                      const std::vector<SoftDollarTier>& tiers) override;
  void familyCodes(const std::vector<FamilyCode>& familyCodes) override;
  void symbolSamples(
      int reqId,
      const std::vector<ContractDescription>& contractDescriptions) override;
  void mktDepthExchanges(const std::vector<DepthMktDataDescription>&
                             depthMktDataDescriptions) override;
  void tickNews(int tickerId, time_t timeStamp, const std::string& providerCode,
                const std::string& articleId, const std::string& headline,
                const std::string& extraData) override;
  void smartComponents(int reqId, const SmartComponentsMap& theMap) override;
  void tickReqParams(int tickerId, double minTick, const std::string& bboExchange,
                    int snapshotPermissions) override;
  void newsProviders(const std::vector<NewsProvider>& newsProviders) override;
  void newsArticle(int requestId, int articleType,
                   const std::string& articleText) override;
  void historicalNews(int requestId, const std::string& time,
                     const std::string& providerCode,
                     const std::string& articleId,
                     const std::string& headline) override;
  void historicalNewsEnd(int requestId, bool hasMore) override;
  void headTimestamp(int reqId, const std::string& headTimestamp) override;
  void histogramData(int reqId, const HistogramDataVector& data) override;
  void historicalDataUpdate(TickerId reqId, const Bar& bar) override;
  void rerouteMktDataReq(int reqId, int conid,
                        const std::string& exchange) override;
  void rerouteMktDepthReq(int reqId, int conid,
                         const std::string& exchange) override;
  void marketRule(int marketRuleId,
                  const std::vector<PriceIncrement>& priceIncrements) override;
  void pnl(int reqId, double dailyPnL, double unrealizedPnL,
           double realizedPnL) override;
  void pnlSingle(int reqId, Decimal pos, double dailyPnL, double unrealizedPnL,
                 double realizedPnL, double value) override;
  void historicalTicks(int reqId, const std::vector<HistoricalTick>& ticks,
                      bool done) override;
  void historicalTicksBidAsk(int reqId,
                            const std::vector<HistoricalTickBidAsk>& ticks,
                            bool done) override;
  void historicalTicksLast(int reqId,
                          const std::vector<HistoricalTickLast>& ticks,
                          bool done) override;
  void tickByTickAllLast(int reqId, int tickType, time_t time, double price,
                        Decimal size, const TickAttribLast& tickAttribLast,
                        const std::string& exchange,
                        const std::string& specialConditions) override;
  void tickByTickBidAsk(int reqId, time_t time, double bidPrice, double askPrice,
                       Decimal bidSize, Decimal askSize,
                       const TickAttribBidAsk& tickAttribBidAsk) override;
  void tickByTickMidPoint(int reqId, time_t time, double midPoint) override;
  void orderBound(long long orderId, int apiClientId, int apiOrderId) override;
  void completedOrder(const Contract& contract, const Order& order,
                      const OrderState& orderState) override;
  void completedOrdersEnd() override;
  void replaceFAEnd(int reqId, const std::string& text) override;
  void wshMetaData(int reqId, const std::string& dataJson) override;
  void wshEventData(int reqId, const std::string& dataJson) override;
  void historicalSchedule(
      int reqId, const std::string& startDateTime,
      const std::string& endDateTime, const std::string& timeZone,
      const std::vector<HistoricalSession>& sessions) override;
  void userInfo(int reqId, const std::string& whiteBrandingId) override;

#if !defined(USE_WIN_DLL)
  void tickPriceProtoBuf(const protobuf::TickPrice& proto) override;
  void tickOptionComputationProtoBuf(
      const protobuf::TickOptionComputation& proto) override;
  void orderStatusProtoBuf(const protobuf::OrderStatus& proto) override;
  void openOrderProtoBuf(const protobuf::OpenOrder& proto) override;
  void execDetailsProtoBuf(const protobuf::ExecutionDetails& proto) override;
  void positionProtoBuf(const protobuf::Position& proto) override;
  void updateAccountValueProtoBuf(const protobuf::AccountValue& proto) override;
  void updatePortfolioProtoBuf(const protobuf::PortfolioValue& proto) override;
  void contractDataProtoBuf(const protobuf::ContractData& proto) override;
  void errorProtoBuf(const protobuf::ErrorMessage& proto) override;
  void historicalDataProtoBuf(const protobuf::HistoricalData& proto) override;
#endif

  // Public API (defined in tws_client.cpp)
  int request_market_data(const types::OptionContract& contract,
                          MarketDataCallback callback);
  void cancel_market_data(int request_id);
  std::optional<types::MarketData> request_market_data_sync(
      const types::OptionContract& contract, int timeout_ms);
  std::vector<types::OptionContract> request_option_chain(
      const std::string& symbol, const std::string& expiry);
  int request_contract_details(const types::OptionContract& contract,
                               ContractDetailsCallback callback);
  long request_contract_details_sync(const types::OptionContract& contract,
                                     int timeout_ms);
  int place_order(const types::OptionContract& contract,
                  types::OrderAction action, int quantity,
                  double limit_price = 0.0,
                  types::TimeInForce tif = types::TimeInForce::Day);
  int place_combo_order(
      const std::vector<types::OptionContract>& contracts,
      const std::vector<types::OrderAction>& actions,
      const std::vector<int>& quantities,
      const std::vector<long>& contract_ids,
      const std::vector<double>& limit_prices,
      types::TimeInForce tif = types::TimeInForce::Day);
  void cancel_order(int order_id);
  void cancel_all_orders();
  std::optional<types::Order> get_order(int order_id) const;
  std::vector<types::Order> get_active_orders() const;
  void request_positions(PositionCallback callback);
  std::vector<types::Position> request_positions_sync(int timeout_ms = 5000);
  std::vector<types::Position> get_positions() const;
  std::optional<types::Position> get_position(
      const types::OptionContract& contract) const;
  void request_account_updates(AccountCallback callback);
  std::optional<types::AccountInfo> request_account_info_sync(
      int timeout_ms = 5000);
  std::optional<types::AccountInfo> get_account_info() const;
  std::optional<types::BoxSpreadLeg> query_box_spread_margin(
      types::BoxSpreadLeg spread, double underlying_price,
      double implied_volatility = 0.20);
  double get_margin_utilization() const;
  bool is_margin_call_risk(double buffer_percent = 10.0) const;
  double get_remaining_margin_capacity() const;
  void set_order_status_callback(OrderStatusCallback callback);
  void set_error_callback(ErrorCallback callback);
  void set_market_data_cache(platform::CacheClient* cache, int ttl_seconds);
  platform::CacheClient* market_data_cache() const {
    return market_data_cache_;
  }
  int market_data_cache_ttl() const { return market_data_cache_ttl_; }
  int get_next_order_id() const;
  bool is_market_open() const;
  std::chrono::system_clock::time_point get_server_time() const;
  void enable_rate_limiting();
  void configure_rate_limiter(const RateLimiterConfig& config);
  std::optional<RateLimiterStatus> get_rate_limiter_status() const;
  void cleanup_stale_rate_limiter_requests(std::chrono::seconds max_age);
  std::pair<std::string, int> get_last_error() const;

 private:
  void start_reader_thread();
  bool wait_for_connection_with_progress(int timeout_ms);
  void seed_mock_state();
  // Stub implementations to fix vtable - the real implementations are in
  // tws_client.cpp but have class splitting issues causing linker errors
  void start_health_monitoring() {}
  void stop_health_monitoring() {}
  void attempt_reconnect_with_backoff() {}

  Contract convert_to_tws_contract(const types::OptionContract& contract);
  inline types::OptionContract convert_from_tws_contract(const Contract& contract) {
    types::OptionContract c;
    c.symbol = contract.symbol;
    c.exchange = contract.exchange;
    c.expiry = contract.lastTradeDateOrContractMonth;
    c.strike = contract.strike;
    c.type = (contract.right == "C") ? types::OptionType::Call
                                     : types::OptionType::Put;
    return c;
  }
  Order create_tws_order(types::OrderAction action, int quantity,
                         double limit_price, types::TimeInForce tif);

  config::TWSConfig config_;
  EReaderOSSignal signal_;
  EClientSocket client_;

  std::atomic<bool> connected_;
  std::atomic<int> next_order_id_;
  std::atomic<int> next_request_id_;
  ConnectionState state_;
  std::atomic<int> last_error_code_{0};
  std::string last_error_message_;
  mutable std::mutex error_mutex_;

  // Reconnection state
  std::atomic<int> reconnect_attempts_{0};
  std::chrono::steady_clock::time_point last_reconnect_attempt_;
  std::mutex reconnect_mutex_;

  // Connection health monitoring
  std::chrono::steady_clock::time_point last_heartbeat_;
  std::chrono::steady_clock::time_point last_message_time_;
  std::atomic<bool> health_check_enabled_{false};
  std::unique_ptr<std::thread> health_check_thread_;

  // Connection callback tracking (for diagnostics)
  struct ConnectionCallbacks {
    bool connectAck = false;
    bool managedAccounts = false;
    std::chrono::steady_clock::time_point connectAck_time;
    std::chrono::steady_clock::time_point managedAccounts_time;
  } connection_callbacks_received_;

  std::mutex reader_mutex_;
  std::unique_ptr<std::thread> reader_thread_;

  // PCAP capture for debugging
  std::unique_ptr<pcap::PcapCapture> pcap_capture_;
  uint32_t local_ip_;    // Cached local IP for pcap
  uint32_t remote_ip_;   // Cached remote IP for pcap
  uint16_t local_port_;  // Cached local port for pcap
  uint16_t remote_port_; // Cached remote port (TWS port) for pcap

  // Connection synchronization
  mutable std::mutex connection_mutex_;
  std::condition_variable connection_cv_;

  // Market data
  mutable std::mutex data_mutex_;
  std::map<int, types::MarketData> market_data_;
  std::map<int, MarketDataCallback> market_data_callbacks_;
  // Synchronous request tracking
  std::map<int, std::shared_ptr<std::promise<types::MarketData>>>
      market_data_promises_;
  // Map request_id (tickerId) to contract symbol for NATS publishing
  std::map<int, std::string> ticker_to_symbol_;

  // Optional cache for TickPrice/TickSize/TickOptionComputation per reqId (ib:tick:<reqId>)
  platform::CacheClient* market_data_cache_{nullptr};
  int market_data_cache_ttl_{60};

  // Contract details
  mutable std::mutex contract_details_mutex_;
  std::map<int, ContractDetailsCallback> contract_details_callbacks_;
  std::map<int, long> contract_details_results_; // Store conId by request ID
  std::map<int, std::shared_ptr<std::promise<long>>> contract_details_promises_;

  // Option chain data
  mutable std::mutex option_chain_mutex_;
  std::map<int, std::set<std::string>>
      option_chain_expirations_; // Store expirations by request ID
  std::map<int, std::set<double>>
      option_chain_strikes_; // Store strikes by request ID
  std::map<int, std::string>
      option_chain_symbols_; // Store symbol by request ID
  std::map<int,
           std::shared_ptr<std::promise<std::vector<types::OptionContract>>>>
      option_chain_promises_;
  std::atomic<bool> option_chain_complete_{false};

  // Orders
  mutable std::mutex order_mutex_;
  std::map<int, types::Order> orders_;
  OrderStatusCallback order_status_callback_;

  // Positions
  mutable std::mutex position_mutex_;
  std::vector<types::Position> positions_;
  PositionCallback position_callback_;
  std::shared_ptr<std::promise<std::vector<types::Position>>>
      positions_promise_;
  std::atomic<bool> positions_request_pending_{false};

  // Account
  mutable std::mutex account_mutex_;
  types::AccountInfo account_info_;
  AccountCallback account_callback_;
  std::shared_ptr<std::promise<types::AccountInfo>> account_promise_;
  std::atomic<bool> account_request_pending_{false};

  // Callbacks
  ErrorCallback error_callback_;

  // Server time (from reqCurrentTime callback)
  std::atomic<long> server_time_epoch_{0};

  // Error tracking
  std::chrono::system_clock::time_point last_error_time_{};
  int error_count_last_hour_{0};

  // Rate limiting
  RateLimiter rate_limiter_;
  bool mock_mode_ = false;

  // NATS client (optional, for message queue integration)
#ifdef ENABLE_NATS
  std::unique_ptr<nats::NatsClient> nats_client_;
#endif
};

}  // namespace tws
