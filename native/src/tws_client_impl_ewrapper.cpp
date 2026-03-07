// tws_client_impl_ewrapper.cpp - EWrapper callback definitions for TWSClient::Impl (split from tws_client.cpp)
// T-1772887500608268454
#include "tws_client_impl.h"
#include "tws_client_error_guidance.h"
#include <spdlog/spdlog.h>

namespace tws {

void TWSClient::Impl::connectAck() {
    try {
      if (config_.log_raw_messages) {
        spdlog::trace("[RAW API] ← connectAck() callback received");
      }
      spdlog::info("✓ connectAck received - Socket connection established, "
                   "server version received");
      spdlog::info(
          "Connection sequence: connectAck → managedAccounts → nextValidId");
      spdlog::info("Status: Waiting for managedAccounts and nextValidId...");

      // Track that we received connectAck (for diagnostics)
      connection_callbacks_received_.connectAck = true;
      connection_callbacks_received_.connectAck_time =
          std::chrono::steady_clock::now();

      // Capture connection event in PCAP
      if (pcap_capture_ && pcap_capture_->is_open()) {
        std::string event_data = "CONNECTION_ACK";
        std::vector<uint8_t> data(event_data.begin(), event_data.end());
        pcap_capture_->capture_raw(data, false, local_port_, remote_port_);
      }

      // In async mode, connectAck is called immediately after socket connection
      // We still need to wait for nextValidId to confirm full connection

      // Reset reconnection attempts on successful connection
      reconnect_attempts_ = 0;
      last_message_time_ = std::chrono::steady_clock::now();

      // Note: Don't start health monitoring yet - wait for nextValidId
      // Request next valid order ID (this will trigger nextValidId callback)
      // According to IB API Quick Reference, reqIds(-1) requests the next valid
      // order ID
      if (config_.log_raw_messages) {
        spdlog::trace(
            "[RAW API] → reqIds(-1) - Requesting next valid order ID");
      }
      client_.reqIds(-1);
      spdlog::debug("Requested next valid order ID via reqIds(-1), waiting for "
                    "nextValidId callback...");
    } catch (const std::exception &e) {
      spdlog::warn("Exception in connectAck: {}", e.what());
    } catch (...) {
      spdlog::warn("Unknown exception in connectAck");
    }
  }

void TWSClient::Impl::connectionClosed() {
    try {
      spdlog::warn("Connection closed by TWS");
      connected_ = false;
      state_ = ConnectionState::Disconnected;
      signal_.issueSignal(); // Wake reader thread so it exits and can be joined
                             // before reconnect

      // Capture disconnection event in PCAP
      if (pcap_capture_ && pcap_capture_->is_open()) {
        std::string event_data = "CONNECTION_CLOSED";
        std::vector<uint8_t> data(event_data.begin(), event_data.end());
        pcap_capture_->capture_raw(data, false, local_port_, remote_port_);
        pcap_capture_->flush();
      }

      // Call error callback
      if (error_callback_) {
        error_callback_(1100, "Connection closed by TWS");
      }

      // Auto-reconnect if enabled with exponential backoff
      if (config_.auto_reconnect) {
        attempt_reconnect_with_backoff();
      }
    } catch (const std::exception &e) {
      spdlog::warn("Exception in connectionClosed: {}", e.what());
    } catch (...) {
      spdlog::warn("Unknown exception in connectionClosed");
    }
  }

  // managedAccounts is called with the list of accounts after connection
  // This is an early indicator of successful connection (happens before
  // nextValidId) According to IB API Quick Reference, this is called after
  // connectAck
void TWSClient::Impl::managedAccounts(const std::string &accountsList) {
    try {
      if (config_.log_raw_messages) {
        spdlog::trace("[RAW API] ← managedAccounts(accountsList=\"{}\") "
                      "callback received",
                      accountsList);
      }
      spdlog::info("✓ managedAccounts received: {} - Connection progressing",
                   accountsList);

      // Track that we received managedAccounts (for diagnostics)
      connection_callbacks_received_.managedAccounts = true;
      connection_callbacks_received_.managedAccounts_time =
          std::chrono::steady_clock::now();

      auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(
                         connection_callbacks_received_.managedAccounts_time -
                         connection_callbacks_received_.connectAck_time)
                         .count();

      // Show progress indicator
      spdlog::info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
                   "━━━━━━━━━━━━━━━━━━━");
      spdlog::info("Connection Progress: [████████░░░░░░░░░░] 50%");
      spdlog::info("  ✓ Step 1/3: connectAck received (socket connected)");
      spdlog::info(
          "  ✓ Step 2/3: managedAccounts received ({}ms after connectAck)",
          elapsed);
      spdlog::info("  ⏳ Step 3/3: Waiting for nextValidId... (this may take a "
                   "few seconds)");
      spdlog::info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
                   "━━━━━━━━━━━━━━━━━━━");

      // Parse account list if needed
      if (!accountsList.empty()) {
        spdlog::info("Account(s) available: {}", accountsList);
        // Store account info for later use
        // Note: This happens before nextValidId, so connection is progressing
      } else {
        spdlog::warn("⚠️  managedAccounts received but account list is empty");
      }
    } catch (const std::exception &e) {
      spdlog::warn("Exception in managedAccounts: {} (accountsList: {})",
                    e.what(), accountsList);
    } catch (...) {
      spdlog::warn("Unknown exception in managedAccounts");
    }
  }

void TWSClient::Impl::nextValidId(OrderId orderId) {
    try {
      if (config_.log_raw_messages) {
        spdlog::trace("[RAW API] ← nextValidId(orderId={}) callback received",
                      orderId);
      }
      // Calculate total connection time
      auto total_elapsed =
          std::chrono::duration_cast<std::chrono::milliseconds>(
              std::chrono::steady_clock::now() -
              connection_callbacks_received_.connectAck_time)
              .count();

      spdlog::info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
                   "━━━━━━━━━━━━━━━━━━━");
      spdlog::info("Connection Progress: "
                   "[████████████████████████████████████████████████] 100%");
      spdlog::info("  ✓ Step 1/3: connectAck received");
      spdlog::info("  ✓ Step 2/3: managedAccounts received");
      spdlog::info(
          "  ✓ Step 3/3: nextValidId received: {} (connection complete!)",
          orderId);
      spdlog::info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
                   "━━━━━━━━━━━━━━━━━━━");
      spdlog::info("✓ Connection fully established in {}ms", total_elapsed);

      next_order_id_ = orderId;

      // Capture connection completion event in PCAP
      if (pcap_capture_ && pcap_capture_->is_open()) {
        std::string event_data =
            "CONNECTION_COMPLETE:nextValidId=" + std::to_string(orderId);
        std::vector<uint8_t> data(event_data.begin(), event_data.end());
        pcap_capture_->capture_raw(data, false, local_port_, remote_port_);
      }

      std::lock_guard<std::mutex> lock(connection_mutex_);
      connected_ = true;

      // Check if this is a reconnection
      bool is_reconnection = (reconnect_attempts_.load() > 0);

      if (is_reconnection) {
        spdlog::info("Reconnection detected ({} attempts). Synchronizing state "
                     "with TWS...",
                     reconnect_attempts_.load());
      }

      // Start health monitoring now that we're fully connected
      start_health_monitoring();

      // According to EWrapper best practices, request open orders to sync state
      // This is important for order recovery after reconnection
      // TWS will call openOrder() for each open order, then openOrderEnd()
      spdlog::debug("Requesting open orders to sync state...");
      client_.reqAllOpenOrders();

      // Sync positions and account after reconnection
      if (is_reconnection) {
        spdlog::debug(
            "Synchronizing positions and account data after reconnection...");
        client_.reqPositions();
        client_.reqAccountUpdates(true, "");
        reconnect_attempts_ = 0; // Reset after successful sync
        spdlog::info("State synchronization complete after reconnection");
      }

      connection_cv_.notify_all();
      spdlog::info("✓ Connection fully established and ready");
    } catch (const std::exception &e) {
      spdlog::warn("Exception in nextValidId: {} (orderId: {})", e.what(),
                    orderId);
    } catch (...) {
      spdlog::warn("Unknown exception in nextValidId (orderId: {})", orderId);
    }
  }

  // ========================================================================
  // EWrapper Callbacks - Market Data
  // ========================================================================

void TWSClient::Impl::tickPrice(TickerId tickerId, TickType field, double price,
                 const TickAttrib &attribs) {
    try {
      if (config_.log_raw_messages) {
        spdlog::trace("[RAW API] ← tickPrice(tickerId={}, field={}, price={}, "
                      "attribs.canAutoExecute={}, "
                      "attribs.pastLimit={}, attribs.preOpen={})",
                      tickerId, static_cast<int>(field), price,
                      attribs.canAutoExecute, attribs.pastLimit,
                      attribs.preOpen);
      }
      spdlog::trace("tickPrice: id={}, field={}, price={}", tickerId,
                    static_cast<int>(field), price);

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
      case HIGH:
        market_data.high = price;
        break;
      case LOW:
        market_data.low = price;
        break;
      case CLOSE:
        market_data.close = price;
        break;
      case OPEN:
        market_data.open = price;
        break;
      default:
        break;
      }

      market_data.timestamp = std::chrono::system_clock::now();

      // Publish to NATS if both bid and ask are available
#ifdef ENABLE_NATS
      if (nats_client_ && nats_client_->is_connected() && field == ASK &&
          market_data.bid > 0.0 && market_data.ask > 0.0) {
        // Get symbol from ticker mapping
        std::string symbol = "UNKNOWN";
        {
          std::lock_guard<std::mutex> lock(data_mutex_);
          if (ticker_to_symbol_.count(tickerId)) {
            symbol = ticker_to_symbol_[tickerId];
          }
        }
        // Format timestamp as ISO 8601
        auto time_t =
            std::chrono::system_clock::to_time_t(market_data.timestamp);
        std::tm tm_buf;
        gmtime_r(&time_t, &tm_buf);
        char timestamp_str[32];
        std::strftime(timestamp_str, sizeof(timestamp_str),
                      "%Y-%m-%dT%H:%M:%SZ", &tm_buf);
        nats_client_->publish_market_data(symbol, market_data.bid,
                                          market_data.ask, timestamp_str);
      }
#endif

      // Notify callback if registered
      if (market_data_callbacks_.count(tickerId)) {
        market_data_callbacks_[tickerId](market_data);
      }

      // Fulfill promise if waiting for synchronous request
      if (market_data_promises_.count(tickerId)) {
        market_data_promises_[tickerId]->set_value(market_data);
        market_data_promises_.erase(tickerId);
      }
    } catch (const std::exception &e) {
      spdlog::warn("Exception in tickPrice(tickerId={}, field={}): {}",
                    tickerId, static_cast<int>(field), e.what());
    } catch (...) {
      spdlog::warn("Unknown exception in tickPrice(tickerId={}, field={})",
                    tickerId, static_cast<int>(field));
    }
  }

void TWSClient::Impl::tickSize(TickerId tickerId, TickType field, Decimal size) {
    try {
      if (config_.log_raw_messages) {
        spdlog::trace("[RAW API] ← tickSize(tickerId={}, field={}, size={})",
                      tickerId, static_cast<int>(field), size);
      }
      spdlog::trace("tickSize: id={}, field={}, size={}", tickerId,
                    static_cast<int>(field), size);

      std::lock_guard<std::mutex> lock(data_mutex_);
      auto &market_data = market_data_[tickerId];

      switch (field) {
      case BID_SIZE:
      case DELAYED_BID_SIZE:
        market_data.bid_size = static_cast<int>(size);
        break;
      case ASK_SIZE:
      case DELAYED_ASK_SIZE:
        market_data.ask_size = static_cast<int>(size);
        break;
      case LAST_SIZE:
      case DELAYED_LAST_SIZE: // TWS API 10.44+: returns Decimal instead of int
        market_data.last_size = static_cast<int>(size);
        break;
      case VOLUME:
      case DELAYED_VOLUME:
        market_data.volume = static_cast<double>(size);
        break;
      default:
        break;
      }
    } catch (const std::exception &e) {
      spdlog::warn("Exception in tickSize(tickerId={}, field={}): {}",
                    tickerId, static_cast<int>(field), e.what());
    } catch (...) {
      spdlog::warn("Unknown exception in tickSize(tickerId={}, field={})",
                    tickerId, static_cast<int>(field));
    }
  }

void TWSClient::Impl::tickOptionComputation(TickerId tickerId, TickType tickType,
                             int tickAttrib, double impliedVol, double delta,
                             double optPrice, double pvDividend, double gamma,
                             double vega, double theta,
                             double undPrice) {
    try {
      spdlog::trace("tickOptionComputation: id={}, IV={}, delta={}", tickerId,
                    impliedVol, delta);

      std::lock_guard<std::mutex> lock(data_mutex_);
      auto &market_data = market_data_[tickerId];

      if (impliedVol >= 0 && impliedVol != DBL_MAX) {
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
    } catch (const std::exception &e) {
      spdlog::warn(
          "Exception in tickOptionComputation(tickerId={}, type={}): {}",
          tickerId, static_cast<int>(tickType), e.what());
    } catch (...) {
      spdlog::warn(
          "Unknown exception in tickOptionComputation(tickerId={}, type={})",
          tickerId, static_cast<int>(tickType));
    }
  }

  // ========================================================================
  // EWrapper Callbacks - Orders
  // ========================================================================

void TWSClient::Impl::orderStatus(OrderId orderId, const std::string &status, Decimal filled,
                   Decimal remaining, double avgFillPrice, long long permId,
                   int parentId, double lastFillPrice, int clientId,
                   const std::string &whyHeld, double mktCapPrice) {
    try {
      spdlog::info("Order #{} status: {}, filled={}, remaining={}, avgPrice={}",
                   orderId, status, filled, remaining, avgFillPrice);

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
        } else if (filled > 0 && remaining > 0) {
          order.status = types::OrderStatus::PartiallyFilled;
        }

        order.filled_quantity = static_cast<int>(filled);
        order.avg_fill_price = avgFillPrice;
        order.last_update = std::chrono::system_clock::now();

        // Notify callback
        if (order_status_callback_) {
          order_status_callback_(order);
        }
      }
    } catch (const std::exception &e) {
      spdlog::warn("Exception in orderStatus(orderId={}, status={}): {}",
                    orderId, status, e.what());
    } catch (...) {
      spdlog::warn("Unknown exception in orderStatus(orderId={}, status={})",
                    orderId, status);
    }
  }

void TWSClient::Impl::openOrder(OrderId orderId, const Contract &contract, const Order &order,
                 const OrderState &orderState) {
    try {
      spdlog::debug("Open order: #{}, {}, {}, status={}", orderId,
                    contract.symbol, order.action, orderState.status);

      std::lock_guard<std::mutex> lock(order_mutex_);

      // openOrder is called for ALL open orders, including ones we didn't place
      // This is important for order recovery and syncing with TWS state

      // Update order if it exists in our tracking
      if (orders_.count(orderId)) {
        auto &our_order = orders_[orderId];

        // Update status based on orderState
        // Note: OrderState contains status string, but filled quantity and avg
        // price come from orderStatus() callback, not openOrder()
        if (orderState.status == "PreSubmitted" ||
            orderState.status == "Submitted") {
          our_order.status = types::OrderStatus::Submitted;
        } else if (orderState.status == "Filled") {
          our_order.status = types::OrderStatus::Filled;
          // Filled quantity and price come from orderStatus() callback
        } else if (orderState.status == "Cancelled") {
          our_order.status = types::OrderStatus::Cancelled;
        } else if (orderState.status == "PartiallyFilled") {
          our_order.status = types::OrderStatus::PartiallyFilled;
          // Filled quantity and price come from orderStatus() callback
        }

        our_order.last_update = std::chrono::system_clock::now();

        // Notify callback
        if (order_status_callback_) {
          order_status_callback_(our_order);
        }
      } else {
        // Order not in our tracking - might be from previous session or another
        // client
        spdlog::debug("Received openOrder for order #{} not in our tracking "
                      "(may be from previous session)",
                      orderId);
      }
    } catch (const std::exception &e) {
      spdlog::warn("Exception in openOrder(orderId={}, symbol={}): {}",
                    orderId, contract.symbol, e.what());
    } catch (...) {
      spdlog::warn("Unknown exception in openOrder(orderId={}, symbol={})",
                    orderId, contract.symbol);
    }
  }

void TWSClient::Impl::openOrderEnd() {
    try {
      spdlog::debug("Open orders sync complete");
      // All open orders have been sent via openOrder() callbacks
      // This is useful for order recovery after reconnection
    } catch (const std::exception &e) {
      spdlog::warn("Exception in openOrderEnd: {}", e.what());
    } catch (...) {
      spdlog::warn("Unknown exception in openOrderEnd");
    }
  }

void TWSClient::Impl::execDetails(int reqId, const Contract &contract,
                   const Execution &execution) {
    try {
      spdlog::info("Execution: order={}, shares={}, price={}, time={}",
                   execution.orderId, execution.shares, execution.price,
                   execution.time);

      std::lock_guard<std::mutex> lock(order_mutex_);

      if (orders_.count(execution.orderId)) {
        auto &order = orders_[execution.orderId];
        order.filled_quantity += static_cast<int>(execution.shares);
        order.avg_fill_price = execution.price;
        order.last_update = std::chrono::system_clock::now();
      }
    } catch (const std::exception &e) {
      spdlog::warn("Exception in execDetails: {} (reqId: {}, orderId: {})",
                    e.what(), reqId, execution.orderId);
    } catch (...) {
      spdlog::warn("Unknown exception in execDetails (reqId: {}, orderId: {})",
                    reqId, execution.orderId);
    }
  }

void TWSClient::Impl::execDetailsEnd(int reqId) {
    try {
      spdlog::debug("Execution details end for reqId={}", reqId);
    } catch (const std::exception &e) {
      spdlog::warn("Exception in execDetailsEnd: {} (reqId: {})", e.what(),
                    reqId);
    } catch (...) {
      spdlog::warn("Unknown exception in execDetailsEnd (reqId: {})", reqId);
    }
  }

  // ========================================================================
  // EWrapper Callbacks - Account & Positions
  // ========================================================================

void TWSClient::Impl::position(const std::string &account, const Contract &contract,
                Decimal position, double avgCost) {
    try {
      spdlog::debug("Position: {} {} @ {} (account={})", position,
                    contract.symbol, avgCost, account);

      std::lock_guard<std::mutex> lock(position_mutex_);

      types::Position pos;
      pos.contract = convert_from_tws_contract(contract);
      pos.quantity = static_cast<int>(position);
      pos.avg_price = avgCost;
      pos.current_price = avgCost; // Will be updated by market data

      // Update or add position
      auto existing = std::find_if(
          positions_.begin(), positions_.end(),
          [&pos](const types::Position &candidate) {
            return candidate.contract.symbol == pos.contract.symbol &&
                   candidate.contract.expiry == pos.contract.expiry &&
                   candidate.contract.strike == pos.contract.strike &&
                   candidate.contract.type == pos.contract.type;
          });

      if (existing != positions_.end()) {
        *existing = pos;
      } else if (position != 0) {
        positions_.push_back(pos);
      }
    } catch (const std::exception &e) {
      spdlog::warn("Exception in position: {} (account: {}, symbol: {})",
                    e.what(), account, contract.symbol);
    } catch (...) {
      spdlog::warn("Unknown exception in position (account: {}, symbol: {})",
                    account, contract.symbol);
    }
  }

void TWSClient::Impl::positionEnd() {
    try {
      spdlog::debug("Position updates complete");

      // Fulfill promise if waiting for synchronous request
      if (positions_request_pending_.load()) {
        std::lock_guard<std::mutex> lock(position_mutex_);
        if (positions_promise_) {
          positions_promise_->set_value(positions_);
          positions_promise_.reset();
          positions_request_pending_ = false;
        }
      }
    } catch (const std::exception &e) {
      spdlog::warn("Exception in positionEnd: {}", e.what());
    } catch (...) {
      spdlog::warn("Unknown exception in positionEnd");
    }
  }

void TWSClient::Impl::updateAccountValue(const std::string &key, const std::string &val,
                          const std::string &currency,
                          const std::string &accountName) {
    try {
      spdlog::trace("Account update: {}={} ({}, {})", key, val, currency,
                    accountName);

      std::lock_guard<std::mutex> lock(account_mutex_);

      try {
        if (key == "NetLiquidation" && currency == "USD") {
          account_info_.net_liquidation = std::stod(val);
        } else if (key == "TotalCashBalance" && currency == "USD") {
          account_info_.cash_balance = std::stod(val);
        } else if (key == "BuyingPower") {
          account_info_.buying_power = std::stod(val);
        } else if (key == "GrossPositionValue" && currency == "USD") {
          account_info_.gross_position_value = std::stod(val);
        } else if (key == "UnrealizedPnL" && currency == "USD") {
          account_info_.unrealized_pnl = std::stod(val);
        } else if (key == "RealizedPnL" && currency == "USD") {
          account_info_.realized_pnl = std::stod(val);
        }
      } catch (const std::exception &e) {
        spdlog::warn("Failed to parse account value: {}={} ({})", key, val,
                     e.what());
      }

      account_info_.account_id = accountName;
      account_info_.timestamp = std::chrono::system_clock::now();
    } catch (const std::exception &e) {
      spdlog::warn(
          "Exception in updateAccountValue: {} (key: {}, account: {})",
          e.what(), key, accountName);
    } catch (...) {
      spdlog::warn(
          "Unknown exception in updateAccountValue (key: {}, account: {})", key,
          accountName);
    }
  }

void TWSClient::Impl::updateAccountTime(const std::string &timeStamp) {
    spdlog::trace("Account time: {}", timeStamp);
  }

void TWSClient::Impl::accountDownloadEnd(const std::string &accountName) {
    spdlog::debug("Account download complete: {}", accountName);

    // Fulfill promise if waiting for synchronous request
    if (account_request_pending_.load()) {
      std::lock_guard<std::mutex> lock(account_mutex_);
      if (account_promise_) {
        account_promise_->set_value(account_info_);
        account_promise_.reset();
        account_request_pending_ = false;
      }
    }
  }

void TWSClient::Impl::updatePortfolio(const Contract &contract, Decimal position,
                       double marketPrice, double marketValue,
                       double averageCost, double unrealizedPNL,
                       double realizedPNL,
                       const std::string &accountName) {
    try {
      spdlog::debug(
          "Portfolio update: {} position={}, value={}, PnL={}, avgCost={}",
          contract.symbol, position, marketValue, unrealizedPNL, averageCost);

      // updatePortfolio is called for each position in the portfolio
      // This is important for tracking current positions and P&L
      // Per EWrapper best practices, this provides real-time position updates

      std::lock_guard<std::mutex> lock(position_mutex_);

      auto it = std::find_if(
          positions_.begin(), positions_.end(),
          [&contract](const types::Position &candidate) {
            return candidate.contract.symbol == contract.symbol &&
                   candidate.contract.expiry ==
                       contract.lastTradeDateOrContractMonth &&
                   std::abs(candidate.contract.strike - contract.strike) < 0.01;
          });

      if (it != positions_.end()) {
        // Update existing position with latest data
        it->current_price = marketPrice;
        it->quantity = static_cast<int>(position);
        it->avg_price = averageCost;
        // Note: unrealizedPNL and realizedPNL could be stored if needed
      } else if (position != 0) {
        // New position - add it (might be from previous session)
        types::Position pos;
        pos.contract = convert_from_tws_contract(contract);
        pos.quantity = static_cast<int>(position);
        pos.avg_price = averageCost;
        pos.current_price = marketPrice;
        positions_.push_back(pos);
        spdlog::debug("Added new position from updatePortfolio: {}",
                      contract.symbol);
      }
    } catch (const std::exception &e) {
      spdlog::warn(
          "Exception in updatePortfolio: {} (symbol: {}, account: {})",
          e.what(), contract.symbol, accountName);
    } catch (...) {
      spdlog::warn(
          "Unknown exception in updatePortfolio (symbol: {}, account: {})",
          contract.symbol, accountName);
    }
  }

  // ========================================================================
  // EWrapper Callbacks - Server Time
  // ========================================================================

void TWSClient::Impl::currentTime(long time) {
    server_time_epoch_.store(time);
    spdlog::debug("TWS server time: {}", time);
  }

  // ========================================================================
  // EWrapper Callbacks - Error Handling
  // ========================================================================

void TWSClient::Impl::error(int id, time_t errorTime, int errorCode,
             const std::string &errorString,
             const std::string &advancedOrderRejectJson) {
    try {
      if (config_.log_raw_messages) {
        spdlog::trace("[RAW API] ← error(id={}, errorTime={}, errorCode={}, "
                      "errorString=\"{}\", "
                      "advancedOrderRejectJson=\"{}\")",
                      id, errorTime, errorCode, errorString,
                      advancedOrderRejectJson);
      }
      // Store error for connection attempt checking
      {
        std::lock_guard<std::mutex> lock(error_mutex_);
        last_error_code_ = errorCode;
        last_error_message_ = errorString;
        last_error_time_ = std::chrono::system_clock::now();
        error_count_last_hour_++;
      }

      std::vector<std::string> guidance_notes;

      // Check for connection failure errors (502 and other connection errors)
      if (errorCode == 502 || (errorCode >= 500 && errorCode < 600)) {
        spdlog::warn("TWS connection error {}: {}", errorCode, errorString);
        connected_ = false;
        state_ = ConnectionState::Error;
        connection_cv_.notify_all(); // Wake up waiting connection attempt
      }

      // Check for authentication/authorization errors
      if (errorCode == 162 || errorCode == 200) {
        spdlog::warn("TWS authentication/authorization error {}: {}",
                      errorCode, errorString);
        spdlog::warn("This usually means:");
        spdlog::warn(
            "  - TWS/Gateway is waiting for you to accept the connection");
        spdlog::warn("  - Check TWS/Gateway window for a connection prompt");
        spdlog::warn("  - Ensure 'Enable ActiveX and Socket Clients' is "
                      "enabled in API settings");
        connected_ = false;
        state_ = ConnectionState::Error;
        connection_cv_.notify_all();
      }

      // Enhanced logging with context and guidance
      std::string guidance = "";
      if (auto it = detail::kIbErrorGuidance.find(errorCode);
          it != detail::kIbErrorGuidance.end()) {
        guidance = it->second;
      }

      // Build context string with order/request details if available
      std::string context = "";
      {
        // Check if ID matches an order ID
        std::lock_guard<std::mutex> lock(order_mutex_);
        if (orders_.count(id) > 0) {
          const auto &order = orders_[id];
          context = "Order #" + std::to_string(id) + ": " +
                    types::order_action_to_string(order.action) + " " +
                    std::to_string(order.quantity) + " " +
                    order.contract.to_string() + " @ " +
                    (order.limit_price > 0 ? std::to_string(order.limit_price)
                                           : "MKT");
        }
      }

      // Check if ID matches a market data request ID
      if (context.empty()) {
        std::lock_guard<std::mutex> lock(data_mutex_);
        if (market_data_.count(id) > 0 ||
            market_data_callbacks_.count(id) > 0) {
          context = "Market data request #" + std::to_string(id);
        }
      }

      // Add advanced order reject JSON context if available
      if (!advancedOrderRejectJson.empty() && advancedOrderRejectJson != "{}") {
        if (!context.empty()) {
          context += " | ";
        }
        context += "Reject JSON: " + advancedOrderRejectJson;
      }

      // Log with appropriate level and context
      if (errorCode >= 2100 && errorCode < 3000) {
        // Informational messages
        if (!context.empty()) {
          spdlog::info("[IB Info {}] ID: {} | {} | Context: {}", errorCode, id,
                       errorString, context);
        } else {
          spdlog::info("[IB Info {}] ID: {} | {}", errorCode, id, errorString);
        }
        if (!guidance.empty()) {
          spdlog::info("  → {}", guidance);
        }
      } else if (errorCode >= 1100 && errorCode < 2000) {
        // System messages
        if (!context.empty()) {
          spdlog::warn("[IB System {}] ID: {} | {} | Context: {}", errorCode,
                       id, errorString, context);
        } else {
          spdlog::warn("[IB System {}] ID: {} | {}", errorCode, id,
                       errorString);
        }
        if (!guidance.empty()) {
          spdlog::warn("  → {}", guidance);
        }

        // Connection lost - trigger reconnection if enabled
        if (errorCode == 1100) {
          connected_ = false;
          state_ = ConnectionState::Error;
          spdlog::warn("Connection lost (error 1100). Auto-reconnect: {}",
                       config_.auto_reconnect ? "enabled" : "disabled");
          if (config_.auto_reconnect) {
            attempt_reconnect_with_backoff();
          }
        }
        // Connection restored
        else if (errorCode == 1101 || errorCode == 1102) {
          connected_ = true;
          state_ = ConnectionState::Connected;
          spdlog::info("Connection restored (error {}). Resuming operations.",
                       errorCode);
        }
      } else {
        // Errors (< 1100)
        if (!context.empty()) {
          spdlog::warn("[IB Error {}] ID: {} | {} | Context: {}", errorCode,
                        id, errorString, context);
        } else {
          spdlog::warn("[IB Error {}] ID: {} | {}", errorCode, id,
                        errorString);
        }
        if (!guidance.empty()) {
          spdlog::warn("  → {}", guidance);
        }
      }

      // Add guidance to notes (already logged above, but keep for callback)
      if (!guidance.empty()) {
        guidance_notes.emplace_back(guidance);
      }

      for (const auto &phrase : detail::kErrorPhraseGuidance) {
        if (errorString.find(phrase.first) != std::string::npos) {
          spdlog::warn("Guidance: {}", phrase.second);
          guidance_notes.emplace_back(phrase.second);
        }
      }

      if (error_callback_) {
        if (guidance_notes.empty()) {
          error_callback_(errorCode, errorString);
        } else {
          std::string enriched_message = errorString + " | Guidance: ";
          for (size_t i = 0; i < guidance_notes.size(); ++i) {
            if (i > 0) {
              enriched_message += " | ";
            }
            enriched_message += guidance_notes[i];
          }
          error_callback_(errorCode, enriched_message);
        }
      }
    } catch (const std::exception &e) {
      spdlog::warn("Exception in error callback: {} (errorCode: {}, id: {})",
                    e.what(), errorCode, id);
    } catch (...) {
      spdlog::warn(
          "Unknown exception in error callback (errorCode: {}, id: {})",
          errorCode, id);
    }
  }

  // ========================================================================
  // Market Data Operations (Public Interface)
  // ========================================================================

  // Note: Most callbacks have default implementations from DefaultEWrapper
  // ========================================================================

void TWSClient::Impl::tickString(TickerId tickerId, TickType tickType,
                  const std::string &value) {}
void TWSClient::Impl::tickEFP(TickerId tickerId, TickType tickType, double basisPoints,
               const std::string &formattedBasisPoints, double totalDividends,
               int holdDays, const std::string &futureLastTradeDate,
               double dividendImpact,
               double dividendsToLastTradeDate) {}
void TWSClient::Impl::tickGeneric(TickerId tickerId, TickType tickType,
                   double value) {}
void TWSClient::Impl::tickSnapshotEnd(int reqId) {}
void TWSClient::Impl::marketDataType(TickerId reqId, int marketDataType) {}
void TWSClient::Impl::realtimeBar(TickerId reqId, long time, double open, double high,
                   double low, double close, Decimal volume, Decimal wap,
                   int count) {}
void TWSClient::Impl::historicalData(TickerId reqId, const Bar &bar) {}
void TWSClient::Impl::historicalDataEnd(int reqId, const std::string &startDateStr,
                         const std::string &endDateStr) {}
void TWSClient::Impl::scannerParameters(const std::string &xml) {}
void TWSClient::Impl::scannerData(int reqId, int rank, const ContractDetails &contractDetails,
                   const std::string &distance, const std::string &benchmark,
                   const std::string &projection,
                   const std::string &legsStr) {}
void TWSClient::Impl::scannerDataEnd(int reqId) {}
void TWSClient::Impl::receiveFA(faDataType pFaDataType, const std::string &cxml) {}
void TWSClient::Impl::bondContractDetails(int reqId,
                           const ContractDetails &contractDetails) {}

void TWSClient::Impl::contractDetails(int reqId,
                       const ContractDetails &contractDetails) {
    try {
      long conId = contractDetails.contract.conId;
      spdlog::debug("Contract details received: reqId={}, conId={}, symbol={}",
                    reqId, conId, contractDetails.contract.symbol);

      ContractDetailsCallback callback;
      std::shared_ptr<std::promise<long>> promise;

      {
        std::lock_guard<std::mutex> lock(contract_details_mutex_);

        // Store result
        contract_details_results_[reqId] = conId;

        // Get callback if registered
        if (contract_details_callbacks_.count(reqId)) {
          callback = contract_details_callbacks_[reqId];
          contract_details_callbacks_.erase(reqId);
        }

        // Get promise if waiting for synchronous request
        if (contract_details_promises_.count(reqId)) {
          promise = contract_details_promises_[reqId];
          contract_details_promises_.erase(reqId);
        }
      }

      // Invoke callback outside lock to avoid deadlock
      if (callback) {
        callback(conId);
      }

      // Fulfill promise outside lock
      if (promise) {
        promise->set_value(conId);
      }
    } catch (const std::exception &e) {
      spdlog::warn("Exception in contractDetails(reqId={}): {}", reqId,
                    e.what());
    } catch (...) {
      spdlog::warn("Unknown exception in contractDetails(reqId={})", reqId);
    }
  }

void TWSClient::Impl::contractDetailsEnd(int reqId) {
    try {
      spdlog::debug("Contract details end for reqId={}", reqId);

      std::lock_guard<std::mutex> lock(contract_details_mutex_);

      // If no result was received, fulfill promise with -1 (failure)
      if (contract_details_promises_.count(reqId)) {
        if (!contract_details_results_.count(reqId)) {
          spdlog::warn("Contract details end without result for reqId={}",
                       reqId);
          contract_details_promises_[reqId]->set_value(-1);
        }
        contract_details_promises_.erase(reqId);
      }

      // Clean up callback if still registered (no results received)
      if (contract_details_callbacks_.count(reqId)) {
        spdlog::warn(
            "Contract details end without result for async callback reqId={}",
            reqId);
        contract_details_callbacks_.erase(reqId);
      }

      // Clean up result
      contract_details_results_.erase(reqId);
    } catch (const std::exception &e) {
      spdlog::warn("Exception in contractDetailsEnd(reqId={}): {}", reqId,
                    e.what());
    } catch (...) {
      spdlog::warn("Unknown exception in contractDetailsEnd(reqId={})", reqId);
    }
  }
void TWSClient::Impl::accountSummary(int reqId, const std::string &account,
                      const std::string &tag, const std::string &value,
                      const std::string &currency) {}
void TWSClient::Impl::accountSummaryEnd(int reqId) {}
void TWSClient::Impl::verifyMessageAPI(const std::string &apiData) {}
void TWSClient::Impl::verifyCompleted(bool isSuccessful,
                       const std::string &errorText) {}
void TWSClient::Impl::verifyAndAuthMessageAPI(const std::string &apiData,
                               const std::string &xyzChallange) {}
void TWSClient::Impl::verifyAndAuthCompleted(bool isSuccessful,
                              const std::string &errorText) {}
void TWSClient::Impl::displayGroupList(int reqId, const std::string &groups) {}
void TWSClient::Impl::displayGroupUpdated(int reqId,
                           const std::string &contractInfo) {}
void TWSClient::Impl::positionMulti(int reqId, const std::string &account,
                     const std::string &modelCode, const Contract &contract,
                     Decimal pos, double avgCost) {}
void TWSClient::Impl::positionMultiEnd(int reqId) {}
void TWSClient::Impl::accountUpdateMulti(int reqId, const std::string &account,
                          const std::string &modelCode, const std::string &key,
                          const std::string &value,
                          const std::string &currency) {}
void TWSClient::Impl::accountUpdateMultiEnd(int reqId) {}
void TWSClient::Impl::securityDefinitionOptionalParameter(
      int reqId, const std::string &exchange, int underlyingConId,
      const std::string &tradingClass, const std::string &multiplier,
      const std::set<std::string> &expirations,
      const std::set<double> &strikes) {
    try {
      spdlog::debug("securityDefinitionOptionalParameter: reqId={}, "
                    "exchange={}, expirations={}, strikes={}",
                    reqId, exchange, expirations.size(), strikes.size());

      std::lock_guard<std::mutex> lock(option_chain_mutex_);

      // Check if this request is being tracked
      if (option_chain_promises_.count(reqId) == 0) {
        spdlog::warn("Option chain callback for unknown request ID: {}", reqId);
        return;
      }

      // Accumulate expirations and strikes (multiple callbacks possible for
      // different exchanges)
      option_chain_expirations_[reqId].insert(expirations.begin(),
                                              expirations.end());
      option_chain_strikes_[reqId].insert(strikes.begin(), strikes.end());

      spdlog::debug("Option chain data accumulated: {} expirations, {} strikes "
                    "for reqId={}",
                    option_chain_expirations_[reqId].size(),
                    option_chain_strikes_[reqId].size(), reqId);
    } catch (const std::exception &e) {
      spdlog::warn(
          "Exception in securityDefinitionOptionalParameter(reqId={}): {}",
          reqId, e.what());
    } catch (...) {
      spdlog::warn(
          "Unknown exception in securityDefinitionOptionalParameter(reqId={})",
          reqId);
    }
  }

void TWSClient::Impl::securityDefinitionOptionalParameterEnd(int reqId) {
    try {
      spdlog::debug("securityDefinitionOptionalParameterEnd: reqId={}", reqId);

      std::lock_guard<std::mutex> lock(option_chain_mutex_);

      // Check if this request is being tracked
      if (option_chain_promises_.count(reqId) == 0) {
        spdlog::warn("Option chain end callback for unknown request ID: {}",
                     reqId);
        return;
      }

      // Get accumulated data
      auto &expirations = option_chain_expirations_[reqId];
      auto &strikes = option_chain_strikes_[reqId];
      std::string symbol = option_chain_symbols_[reqId];

      // Convert expirations and strikes to OptionContract vector
      std::vector<types::OptionContract> contracts;
      for (const auto &exp : expirations) {
        for (double strike : strikes) {
          // Create both call and put contracts
          types::OptionContract call_contract;
          call_contract.symbol = symbol;
          call_contract.expiry = exp;
          call_contract.strike = strike;
          call_contract.type = types::OptionType::Call;
          call_contract.exchange = "SMART";
          call_contract.style =
              types::OptionStyle::American; // Most US options are American
          contracts.push_back(call_contract);

          types::OptionContract put_contract;
          put_contract.symbol = symbol;
          put_contract.expiry = exp;
          put_contract.strike = strike;
          put_contract.type = types::OptionType::Put;
          put_contract.exchange = "SMART";
          put_contract.style = types::OptionStyle::American;
          contracts.push_back(put_contract);
        }
      }

      spdlog::info("Option chain complete: {} contracts for {} (reqId={})",
                   contracts.size(), symbol, reqId);

      // Fulfill promise
      if (option_chain_promises_.count(reqId)) {
        option_chain_promises_[reqId]->set_value(contracts);
      }
    } catch (const std::exception &e) {
      spdlog::warn(
          "Exception in securityDefinitionOptionalParameterEnd(reqId={}): {}",
          reqId, e.what());
      // Fulfill promise with empty vector on error
      std::lock_guard<std::mutex> lock(option_chain_mutex_);
      if (option_chain_promises_.count(reqId)) {
        option_chain_promises_[reqId]->set_value(
            std::vector<types::OptionContract>());
      }
    } catch (...) {
      spdlog::warn("Unknown exception in "
                    "securityDefinitionOptionalParameterEnd(reqId={})",
                    reqId);
      std::lock_guard<std::mutex> lock(option_chain_mutex_);
      if (option_chain_promises_.count(reqId)) {
        option_chain_promises_[reqId]->set_value(
            std::vector<types::OptionContract>());
      }
    }
  }
void TWSClient::Impl::softDollarTiers(int reqId,
                       const std::vector<SoftDollarTier> &tiers) {}
void TWSClient::Impl::familyCodes(const std::vector<FamilyCode> &familyCodes) {}
void TWSClient::Impl::symbolSamples(
      int reqId,
      const std::vector<ContractDescription> &contractDescriptions) {}
void TWSClient::Impl::mktDepthExchanges(const std::vector<DepthMktDataDescription>
                             &depthMktDataDescriptions) {}
void TWSClient::Impl::tickNews(int tickerId, time_t timeStamp, const std::string &providerCode,
                const std::string &articleId, const std::string &headline,
                const std::string &extraData) {}
void TWSClient::Impl::smartComponents(int reqId, const SmartComponentsMap &theMap) {}
void TWSClient::Impl::tickReqParams(int tickerId, double minTick,
                     const std::string &bboExchange,
                     int snapshotPermissions) {}
void TWSClient::Impl::newsProviders(const std::vector<NewsProvider> &newsProviders) {}
void TWSClient::Impl::newsArticle(int requestId, int articleType,
                   const std::string &articleText) {}
void TWSClient::Impl::historicalNews(int requestId, const std::string &time,
                      const std::string &providerCode,
                      const std::string &articleId,
                      const std::string &headline) {}
void TWSClient::Impl::historicalNewsEnd(int requestId, bool hasMore) {}
void TWSClient::Impl::headTimestamp(int reqId, const std::string &headTimestamp) {}
void TWSClient::Impl::histogramData(int reqId, const HistogramDataVector &data) {}
void TWSClient::Impl::historicalDataUpdate(TickerId reqId, const Bar &bar) {}
void TWSClient::Impl::rerouteMktDataReq(int reqId, int conid,
                         const std::string &exchange) {}
void TWSClient::Impl::rerouteMktDepthReq(int reqId, int conid,
                          const std::string &exchange) {}
void TWSClient::Impl::marketRule(int marketRuleId,
                  const std::vector<PriceIncrement> &priceIncrements) {
  }
void TWSClient::Impl::pnl(int reqId, double dailyPnL, double unrealizedPnL,
           double realizedPnL) {}
void TWSClient::Impl::pnlSingle(int reqId, Decimal pos, double dailyPnL, double unrealizedPnL,
                 double realizedPnL, double value) {}
void TWSClient::Impl::historicalTicks(int reqId, const std::vector<HistoricalTick> &ticks,
                       bool done) {}
void TWSClient::Impl::historicalTicksBidAsk(int reqId,
                             const std::vector<HistoricalTickBidAsk> &ticks,
                             bool done) {}
void TWSClient::Impl::historicalTicksLast(int reqId,
                           const std::vector<HistoricalTickLast> &ticks,
                           bool done) {}
void TWSClient::Impl::tickByTickAllLast(int reqId, int tickType, time_t time, double price,
                         Decimal size, const TickAttribLast &tickAttribLast,
                         const std::string &exchange,
                         const std::string &specialConditions) {}
void TWSClient::Impl::tickByTickBidAsk(int reqId, time_t time, double bidPrice,
                        double askPrice, Decimal bidSize, Decimal askSize,
                        const TickAttribBidAsk &tickAttribBidAsk) {}
void TWSClient::Impl::tickByTickMidPoint(int reqId, time_t time, double midPoint) {}
void TWSClient::Impl::orderBound(long long orderId, int apiClientId, int apiOrderId) {
  }
void TWSClient::Impl::completedOrder(const Contract &contract, const Order &order,
                      const OrderState &orderState) {}
void TWSClient::Impl::completedOrdersEnd() {}
void TWSClient::Impl::replaceFAEnd(int reqId, const std::string &text) {}
void TWSClient::Impl::wshMetaData(int reqId, const std::string &dataJson) {}
void TWSClient::Impl::wshEventData(int reqId, const std::string &dataJson) {}
  void
  historicalSchedule(int reqId, const std::string &startDateTime,
                     const std::string &endDateTime,
                     const std::string &timeZone,
                     const std::vector<HistoricalSession> &sessions) {}
void TWSClient::Impl::userInfo(int reqId, const std::string &whiteBrandingId) {}

  // ========================================================================
  // Proto EWrapper Callbacks — structured telemetry (Category A: logging only)
  // EDecoder fires proto callbacks BEFORE the matching legacy callback; never
  // store state here. Guard with log_raw_messages to avoid trace spam in prod.
  // ========================================================================

#if !defined(USE_WIN_DLL)
void TWSClient::Impl::tickPriceProtoBuf(const protobuf::TickPrice &proto)
  {
    if (!config_.log_raw_messages) { return; }
    spdlog::trace("[PROTO] ← TickPrice reqId={} type={} price={} size={}",
                  proto.reqid(), proto.ticktype(), proto.price(), proto.size());
    // Legacy tickPrice() fires immediately after — no data processing here.
    // proto.size() carries bid/ask size bundled with the price tick; extract
    // via DecimalFunctions::stringToDecimal(proto.size()) if needed before
    // the synthetic tickSize fires.
  }

void TWSClient::Impl::tickOptionComputationProtoBuf(
      const protobuf::TickOptionComputation &proto)
  {
    if (!config_.log_raw_messages) { return; }
    spdlog::trace(
        "[PROTO] ← TickOptionComputation reqId={} type={} iv={:.6f} delta={:.6f}",
        proto.reqid(), proto.ticktype(), proto.impliedvol(), proto.delta());
  }

void TWSClient::Impl::orderStatusProtoBuf(const protobuf::OrderStatus &proto)
  {
    if (!config_.log_raw_messages) { return; }
    spdlog::trace(
        "[PROTO] ← OrderStatus orderId={} status={} filled={} remaining={}",
        proto.orderid(), proto.status(), proto.filled(), proto.remaining());
  }

void TWSClient::Impl::openOrderProtoBuf(const protobuf::OpenOrder &proto)
  {
    if (!config_.log_raw_messages) { return; }
    spdlog::trace("[PROTO] ← OpenOrder orderId={} symbol={}",
                  proto.orderid(),
                  proto.has_contract() ? proto.contract().symbol() : "");
  }

void TWSClient::Impl::execDetailsProtoBuf(const protobuf::ExecutionDetails &proto)
  {
    if (!config_.log_raw_messages) { return; }
    spdlog::trace("[PROTO] ← ExecutionDetails reqId={} symbol={} execId={}",
                  proto.reqid(),
                  proto.has_contract() ? proto.contract().symbol() : "",
                  proto.has_execution() ? proto.execution().execid() : "");
  }

void TWSClient::Impl::positionProtoBuf(const protobuf::Position &proto)
  {
    if (!config_.log_raw_messages) { return; }
    spdlog::trace("[PROTO] ← Position account={} symbol={} qty={} avgCost={}",
                  proto.account(),
                  proto.has_contract() ? proto.contract().symbol() : "",
                  proto.position(), proto.avgcost());
  }

void TWSClient::Impl::updateAccountValueProtoBuf(const protobuf::AccountValue &proto)
  {
    if (!config_.log_raw_messages) { return; }
    spdlog::trace(
        "[PROTO] ← AccountValue account={} key={} value={} currency={}",
        proto.accountname(), proto.key(), proto.value(), proto.currency());
  }

void TWSClient::Impl::updatePortfolioProtoBuf(const protobuf::PortfolioValue &proto)
  {
    if (!config_.log_raw_messages) { return; }
    spdlog::trace("[PROTO] ← PortfolioValue account={} symbol={} qty={}",
                  proto.accountname(),
                  proto.has_contract() ? proto.contract().symbol() : "",
                  proto.position());
  }

void TWSClient::Impl::contractDataProtoBuf(const protobuf::ContractData &proto)
  {
    if (!config_.log_raw_messages) { return; }
    spdlog::trace("[PROTO] ← ContractData reqId={} symbol={}",
                  proto.reqid(),
                  proto.has_contract() ? proto.contract().symbol() : "");
  }

void TWSClient::Impl::errorProtoBuf(const protobuf::ErrorMessage &proto)
  {
    if (!config_.log_raw_messages) { return; }
    spdlog::trace("[PROTO] ← ErrorMessage id={} code={} msg={}",
                  proto.id(), proto.errorcode(), proto.errormsg());
  }

  // Category B: all bars arrive atomically — future batch processing goes here
  // instead of the legacy per-bar historicalData() path.
void TWSClient::Impl::historicalDataProtoBuf(const protobuf::HistoricalData &proto)
  {
    if (!config_.log_raw_messages) { return; }
    spdlog::trace("[PROTO] ← HistoricalData reqId={} bars={}",
                  proto.reqid(), proto.historicaldatabars_size());
  }
#endif

}  // namespace tws
