// tws_client.cpp - TWS API Client implementation with full EWrapper integration
#include "tws_client.h"
#include <spdlog/spdlog.h>

// TWS API headers
#include "EWrapper.h"
#include "DefaultEWrapper.h"
#include "EClientSocket.h"
#include "EReaderOSSignal.h"
#include "EReader.h"
#include "Contract.h"
#include "Order.h"
#include "OrderState.h"
#include "Execution.h"
#include "OrderCancel.h"

#include <algorithm>
#include <thread>
#include <mutex>
#include <condition_variable>
#include <atomic>
#include <chrono>
#include <cfloat>
#include <ctime>
#include <unordered_map>
#include <vector>

// NOTE FOR AUTOMATION AGENTS:
// `TWSClient` is the primary integration point with Interactive Brokers' TWS API.
// It wraps the asynchronous `EClientSocket`/`EWrapper` interface, translating IB
// callbacks into thread-safe data structures consumed by higher layers. When
// extending behaviour, focus on the `Impl` class and prefer reusing the helper
// converters and callback registration patterns already established below.

namespace tws {

namespace {

const std::unordered_map<int, std::string> kIbErrorGuidance = {
    {1100, "IB lost connectivity. Check TWS/IB Gateway and internet; re-authenticate if prompted."},
    {1101, "Market data connection restored. Confirm subscriptions are active."},
    {1102, "Order routing connection restored."},
    {200, "Invalid contract definition. Verify symbol, expiry, right, strike, and exchange values."},
    {201, "Order rejected due to contract error. Confirm contract fields before resubmitting."},
    {202, "Order rejected by IB. Check order parameters, size limits, and account permissions."},
    {321, "Server validation failed. Review price increments, exchange routing, and TIF."},
    {354, "No market data permissions. Ensure your IB account has the required data subscriptions."},
    {2104, "Market data farm connection restored."},
    {2106, "Market data farm is connecting. Expect delayed quotes until established."},
    {2107, "Market data farm connection failed. Check IB network status dashboard."},
    {2108, "Market data farm disconnected. Quotes will pause until reconnection."},
    {2109, "Order routing to IB server is OK."},
};

const std::pair<const char*, const char*> kErrorPhraseGuidance[] = {
    {
        "code card authentication",
        "IB triggered code card authentication. Approve the 2FA challenge in IBKR Mobile or disable code card auth.",
    },
    {
        "two factor authentication request timed out",
        "Two-factor approval timed out. Re-initiate login and approve promptly on your IBKR Mobile device.",
    },
    {
        "No market data permissions",
        "IB refused market data. Purchase/enable required market data subscriptions or switch data provider.",
    },
    {
        "No security definition has been found",
        "Contract not recognized. Double-check ticker, expiration, strike, right, and exchange.",
    },
};

}  // namespace

// ============================================================================
// TWSClient::Impl - Full TWS API Implementation with DefaultEWrapper
// ============================================================================

class TWSClient::Impl : public DefaultEWrapper {
public:
    explicit Impl(const config::TWSConfig& config)
        : config_(config)
        , signal_(2000) // 2 second timeout
        , client_(this, &signal_)
        , next_order_id_(0)
        , connected_(false)
        , next_request_id_(1000)
        , state_(ConnectionState::Disconnected) {
    }

    ~Impl() {
        disconnect();
    }

    // ========================================================================
    // Connection Management
    // ========================================================================

    bool connect() {
        spdlog::info("Connecting to TWS at {}:{}...", config_.host, config_.port);

        state_ = ConnectionState::Connecting;

        // Connect to TWS
        bool success = client_.eConnect(
            config_.host.c_str(),
            config_.port,
            config_.client_id
        );

        if (!success) {
            spdlog::error("Failed to connect to TWS");
            state_ = ConnectionState::Error;
            return false;
        }

        // Wait for connection acknowledgment
        if (!wait_for_connection(config_.connection_timeout_ms)) {
            spdlog::error("Connection timeout");
            client_.eDisconnect();
            state_ = ConnectionState::Error;
            return false;
        }

        // Start message reader thread
        start_reader_thread();

        state_ = ConnectionState::Connected;
        spdlog::info("✓ Connected to TWS");
        return true;
    }

    void disconnect() {
        if (connected_) {
            spdlog::info("Disconnecting from TWS...");

            // Stop reader thread
            if (reader_thread_ && reader_thread_->joinable()) {
                connected_ = false;
                signal_.issueSignal();  // Wake up the thread
                reader_thread_->join();
            }

            client_.eDisconnect();
            state_ = ConnectionState::Disconnected;
            spdlog::info("✓ Disconnected");
        }
    }

    bool is_connected() const {
        return connected_ && client_.isConnected();
    }

    ConnectionState get_connection_state() const {
        return state_;
    }

    void process_messages(int timeout_ms) {
        // Messages are processed by the reader thread
        // This method can be used for additional processing if needed
        std::this_thread::sleep_for(std::chrono::milliseconds(timeout_ms));
    }

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
        state_ = ConnectionState::Disconnected;

        // Call error callback
        if (error_callback_) {
            error_callback_(1100, "Connection closed by TWS");
        }

        // Auto-reconnect if enabled
        if (config_.auto_reconnect) {
            std::this_thread::sleep_for(
                std::chrono::milliseconds(config_.reconnect_delay_ms)
            );
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

    void tickPrice(TickerId tickerId, TickType field,
                   double price, const TickAttrib& attribs) override {
        spdlog::trace("tickPrice: id={}, field={}, price={}", tickerId, field, price);

        std::lock_guard<std::mutex> lock(data_mutex_);
        auto& market_data = market_data_[tickerId];

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

        // Notify callback if registered
        if (market_data_callbacks_.count(tickerId)) {
            market_data_callbacks_[tickerId](market_data);
        }
    }

    void tickSize(TickerId tickerId, TickType field, Decimal size) override {
        spdlog::trace("tickSize: id={}, field={}, size={}", tickerId, field, size);

        std::lock_guard<std::mutex> lock(data_mutex_);
        auto& market_data = market_data_[tickerId];

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
                               int tickAttrib, double impliedVol,
                               double delta, double optPrice,
                               double pvDividend, double gamma,
                               double vega, double theta,
                               double undPrice) override {
        spdlog::trace("tickOptionComputation: id={}, IV={}, delta={}",
                     tickerId, impliedVol, delta);

        std::lock_guard<std::mutex> lock(data_mutex_);
        auto& market_data = market_data_[tickerId];

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
    }

    // ========================================================================
    // EWrapper Callbacks - Orders
    // ========================================================================

    void orderStatus(OrderId orderId, const std::string& status,
                    Decimal filled, Decimal remaining,
                    double avgFillPrice, long long permId, int parentId,
                    double lastFillPrice, int clientId,
                    const std::string& whyHeld, double mktCapPrice) override {
        spdlog::info("Order #{} status: {}, filled={}, remaining={}, avgPrice={}",
                    orderId, status, filled, remaining, avgFillPrice);

        std::lock_guard<std::mutex> lock(order_mutex_);

        if (orders_.count(orderId)) {
            auto& order = orders_[orderId];

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
    }

    void openOrder(OrderId orderId, const Contract& contract,
                   const Order& order, const OrderState& orderState) override {
        spdlog::debug("Open order: #{}, {}, {}", orderId, contract.symbol, order.action);

        std::lock_guard<std::mutex> lock(order_mutex_);

        // Update order if it exists
        if (orders_.count(orderId)) {
            auto& our_order = orders_[orderId];
            our_order.status = types::OrderStatus::Submitted;

            if (orderState.status == "Filled") {
                our_order.status = types::OrderStatus::Filled;
            } else if (orderState.status == "Cancelled") {
                our_order.status = types::OrderStatus::Cancelled;
            }
        }
    }

    void execDetails(int reqId, const Contract& contract,
                    const Execution& execution) override {
        spdlog::info("Execution: order={}, shares={}, price={}, time={}",
                    execution.orderId, execution.shares, execution.price, execution.time);

        std::lock_guard<std::mutex> lock(order_mutex_);

        if (orders_.count(execution.orderId)) {
            auto& order = orders_[execution.orderId];
            order.filled_quantity += static_cast<int>(execution.shares);
            order.avg_fill_price = execution.price;
            order.last_update = std::chrono::system_clock::now();
        }
    }

    void execDetailsEnd(int reqId) override {
        spdlog::debug("Execution details end for reqId={}", reqId);
    }

    // ========================================================================
    // EWrapper Callbacks - Account & Positions
    // ========================================================================

    void position(const std::string& account, const Contract& contract,
                 Decimal position, double avgCost) override {
        spdlog::debug("Position: {} {} @ {} (account={})",
                     position, contract.symbol, avgCost, account);

        std::lock_guard<std::mutex> lock(position_mutex_);

        types::Position pos;
        pos.contract = convert_from_tws_contract(contract);
        pos.quantity = static_cast<int>(position);
        pos.avg_price = avgCost;
        pos.current_price = avgCost;  // Will be updated by market data

        // Update or add position
        auto existing = std::find_if(
            positions_.begin(),
            positions_.end(),
            [&pos](const types::Position& candidate) {
                return candidate.contract.symbol == pos.contract.symbol &&
                       candidate.contract.expiry == pos.contract.expiry &&
                       candidate.contract.strike == pos.contract.strike &&
                       candidate.contract.type == pos.contract.type;
            }
        );

        if (existing != positions_.end()) {
            *existing = pos;
        } else if (position != 0) {
            positions_.push_back(pos);
        }
    }

    void positionEnd() override {
        spdlog::debug("Position updates complete");
    }

    void updateAccountValue(const std::string& key, const std::string& val,
                           const std::string& currency,
                           const std::string& accountName) override {
        spdlog::trace("Account update: {}={} ({}, {})", key, val, currency, accountName);

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
        } catch (const std::exception& e) {
            spdlog::warn("Failed to parse account value: {}={} ({})", key, val, e.what());
        }

        account_info_.account_id = accountName;
        account_info_.timestamp = std::chrono::system_clock::now();
    }

    void updateAccountTime(const std::string& timeStamp) override {
        spdlog::trace("Account time: {}", timeStamp);
    }

    void accountDownloadEnd(const std::string& accountName) override {
        spdlog::debug("Account download complete: {}", accountName);
    }

    void updatePortfolio(const Contract& contract, Decimal position,
                        double marketPrice, double marketValue,
                        double averageCost, double unrealizedPNL,
                        double realizedPNL, const std::string& accountName) override {
        spdlog::debug("Portfolio update: {} position={}, value={}, PnL={}",
                     contract.symbol, position, marketValue, unrealizedPNL);

        // Update position with current market price
        std::lock_guard<std::mutex> lock(position_mutex_);

        auto it = std::find_if(
            positions_.begin(),
            positions_.end(),
            [&contract](const types::Position& candidate) {
                return candidate.contract.symbol == contract.symbol &&
                       candidate.contract.expiry ==
                           contract.lastTradeDateOrContractMonth &&
                       std::abs(candidate.contract.strike - contract.strike) < 0.01;
            }
        );

        if (it != positions_.end()) {
            it->current_price = marketPrice;
        }
    }

    // ========================================================================
    // EWrapper Callbacks - Error Handling
    // ========================================================================

    void error(int id, time_t errorTime, int errorCode, const std::string& errorString,
              const std::string& advancedOrderRejectJson) override {
        std::vector<std::string> guidance_notes;

        if (errorCode >= 2100 && errorCode < 3000) {
            // Informational messages
            spdlog::info("TWS message {} (id={}): {}", errorCode, id, errorString);
        } else if (errorCode >= 1100 && errorCode < 2000) {
            // System messages
            spdlog::warn("TWS system {} (id={}): {}", errorCode, id, errorString);

            // Connection lost
            if (errorCode == 1100) {
                connected_ = false;
                state_ = ConnectionState::Error;
            }
            // Connection restored
            else if (errorCode == 1101 || errorCode == 1102) {
                connected_ = true;
                state_ = ConnectionState::Connected;
            }
        } else {
            // Errors
            spdlog::error("TWS error {} (id={}): {}", errorCode, id, errorString);
        }

        if (auto it = kIbErrorGuidance.find(errorCode); it != kIbErrorGuidance.end()) {
            spdlog::warn("Guidance: {}", it->second);
            guidance_notes.emplace_back(it->second);
        }

        for (const auto& phrase : kErrorPhraseGuidance) {
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
    }

    // ========================================================================
    // Market Data Operations (Public Interface)
    // ========================================================================

    int request_market_data(const types::OptionContract& contract,
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
        client_.reqMktData(
            request_id,           // Request ID
            tws_contract,         // Contract
            "",                   // Generic tick list
            false,                // Snapshot
            false,                // Regulatory snapshot
            TagValueListSPtr()    // Options
        );

        spdlog::debug("Requested market data for {} (id={})",
                     contract.to_string(), request_id);

        return request_id;
    }

    void cancel_market_data(int request_id) {
        client_.cancelMktData(request_id);

        std::lock_guard<std::mutex> lock(data_mutex_);
        market_data_callbacks_.erase(request_id);
        market_data_.erase(request_id);

        spdlog::debug("Cancelled market data request {}", request_id);
    }

    std::vector<types::OptionContract> request_option_chain(
        const std::string& symbol,
        const std::string& expiry) {

        spdlog::debug("Requesting option chain for {} (expiry={})",
                     symbol, expiry.empty() ? "all" : expiry);

        // TODO: Implement option chain request using reqSecDefOptParams
        // This is complex and requires handling multiple callbacks
        return {};
    }

    // ========================================================================
    // Order Operations (Public Interface)
    // ========================================================================

    int place_order(const types::OptionContract& contract,
                   types::OrderAction action,
                   int quantity,
                   double limit_price,
                   types::TimeInForce tif) {
        int order_id = next_order_id_++;

        // Convert to TWS types
        Contract tws_contract = convert_to_tws_contract(contract);
        Order tws_order = create_tws_order(action, quantity, limit_price, tif);

        // Place order
        client_.placeOrder(order_id, tws_contract, tws_order);

        spdlog::info("Placed order #{}: {} {} {} @ {}",
                    order_id,
                    types::order_action_to_string(action),
                    quantity,
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
        OrderCancel orderCancel;
        client_.cancelOrder(order_id, orderCancel);
        spdlog::info("Cancelling order #{}", order_id);

        std::lock_guard<std::mutex> lock(order_mutex_);
        if (orders_.count(order_id)) {
            orders_[order_id].status = types::OrderStatus::Cancelled;
            orders_[order_id].last_update = std::chrono::system_clock::now();
        }
    }

    void cancel_all_orders() {
        spdlog::info("Cancelling all orders");
        OrderCancel orderCancel;
        client_.reqGlobalCancel(orderCancel);

        std::lock_guard<std::mutex> lock(order_mutex_);
        for (auto& [id, order] : orders_) {
            if (order.is_active()) {
                order.status = types::OrderStatus::Cancelled;
                order.last_update = std::chrono::system_clock::now();
            }
        }
    }

    std::optional<types::Order> get_order(int order_id) const {
        std::lock_guard<std::mutex> lock(order_mutex_);
        auto it = orders_.find(order_id);
        if (it != orders_.end()) {
            return it->second;
        }
        return std::nullopt;
    }

    std::vector<types::Order> get_active_orders() const {
        std::lock_guard<std::mutex> lock(order_mutex_);
        std::vector<types::Order> active_orders;
        for (const auto& [id, order] : orders_) {
            if (order.is_active()) {
                active_orders.push_back(order);
            }
        }
        return active_orders;
    }

    // ========================================================================
    // Position Operations (Public Interface)
    // ========================================================================

    void request_positions(PositionCallback callback) {
        spdlog::debug("Requesting positions");
        position_callback_ = callback;
        client_.reqPositions();
    }

    std::vector<types::Position> get_positions() const {
        std::lock_guard<std::mutex> lock(position_mutex_);
        return positions_;
    }

    std::optional<types::Position> get_position(
        const types::OptionContract& contract) const {

        std::lock_guard<std::mutex> lock(position_mutex_);
        auto it = std::find_if(
            positions_.begin(),
            positions_.end(),
            [&contract](const types::Position& pos) {
                return pos.contract.symbol == contract.symbol &&
                       pos.contract.expiry == contract.expiry &&
                       std::abs(pos.contract.strike - contract.strike) < 0.01 &&
                       pos.contract.type == contract.type;
            }
        );
        if (it != positions_.end()) {
            return *it;
        }
        return std::nullopt;
    }

    // ========================================================================
    // Account Operations (Public Interface)
    // ========================================================================

    void request_account_updates(AccountCallback callback) {
        spdlog::debug("Requesting account updates");
        account_callback_ = callback;
        client_.reqAccountUpdates(true, "");
    }

    std::optional<types::AccountInfo> get_account_info() const {
        std::lock_guard<std::mutex> lock(account_mutex_);
        if (account_info_.account_id.empty()) {
            return std::nullopt;
        }
        return account_info_;
    }

    // ========================================================================
    // Callbacks
    // ========================================================================

    void set_order_status_callback(OrderStatusCallback callback) {
        order_status_callback_ = callback;
    }

    void set_error_callback(ErrorCallback callback) {
        error_callback_ = callback;
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    int get_next_order_id() const {
        return next_order_id_;
    }

    bool is_market_open() const {
        // TODO: Implement proper market hours check
        auto now = std::chrono::system_clock::now();
        auto time_t_now = std::chrono::system_clock::to_time_t(now);
        auto tm_now = *std::localtime(&time_t_now);

        // Simple check: weekday and between 9:30 AM and 4:00 PM ET
        // This is approximate and doesn't account for holidays
        int hour = tm_now.tm_hour;
        int wday = tm_now.tm_wday;

        return (wday >= 1 && wday <= 5) && (hour >= 9 && hour < 16);
    }

    std::chrono::system_clock::time_point get_server_time() const {
        // TODO: Request and return actual TWS server time
        return std::chrono::system_clock::now();
    }

private:
    // ========================================================================
    // Helper Methods - Private
    // ========================================================================

    void start_reader_thread() {
        auto reader = std::make_unique<EReader>(&client_, &signal_);
        reader->start();

        reader_thread_ = std::make_unique<std::thread>([this, r = std::move(reader)]() mutable {
            while (connected_) {
                signal_.waitForSignal();
                if (!connected_) break;

                try {
                    r->processMsgs();
                } catch (const std::exception& e) {
                    spdlog::error("Error processing messages: {}", e.what());
                }
            }
        });
    }

    bool wait_for_connection(int timeout_ms) {
        std::unique_lock<std::mutex> lock(connection_mutex_);
        return connection_cv_.wait_for(
            lock,
            std::chrono::milliseconds(timeout_ms),
            [this] { return connected_.load(); }
        );
    }

    Contract convert_to_tws_contract(const types::OptionContract& contract) {
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

    types::OptionContract convert_from_tws_contract(const Contract& contract) {
        types::OptionContract c;
        c.symbol = contract.symbol;
        c.exchange = contract.exchange;
        c.expiry = contract.lastTradeDateOrContractMonth;
        c.strike = contract.strike;
        c.type = (contract.right == "C") ? types::OptionType::Call : types::OptionType::Put;
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
    // Remaining EWrapper callbacks (stubs - implement as needed)
    // Note: Most callbacks have default implementations from DefaultEWrapper
    // ========================================================================

    void tickString(TickerId tickerId, TickType tickType,
                   const std::string& value) override {}
    void tickEFP(TickerId tickerId, TickType tickType, double basisPoints,
                const std::string& formattedBasisPoints, double totalDividends,
                int holdDays, const std::string& futureLastTradeDate,
                double dividendImpact, double dividendsToLastTradeDate) override {}
    void tickGeneric(TickerId tickerId, TickType tickType, double value) override {}
    void tickSnapshotEnd(int reqId) override {}
    void marketDataType(TickerId reqId, int marketDataType) override {}
    void realtimeBar(TickerId reqId, long time, double open, double high,
                    double low, double close, Decimal volume, Decimal wap,
                    int count) override {}
    void historicalData(TickerId reqId, const Bar& bar) override {}
    void historicalDataEnd(int reqId, const std::string& startDateStr,
                          const std::string& endDateStr) override {}
    void scannerParameters(const std::string& xml) override {}
    void scannerData(int reqId, int rank, const ContractDetails& contractDetails,
                    const std::string& distance, const std::string& benchmark,
                    const std::string& projection, const std::string& legsStr) override {}
    void scannerDataEnd(int reqId) override {}
    void receiveFA(faDataType pFaDataType, const std::string& cxml) override {}
    void bondContractDetails(int reqId, const ContractDetails& contractDetails) override {}
    void contractDetails(int reqId, const ContractDetails& contractDetails) override {}
    void contractDetailsEnd(int reqId) override {}
    void accountSummary(int reqId, const std::string& account, const std::string& tag,
                       const std::string& value, const std::string& currency) override {}
    void accountSummaryEnd(int reqId) override {}
    void verifyMessageAPI(const std::string& apiData) override {}
    void verifyCompleted(bool isSuccessful, const std::string& errorText) override {}
    void verifyAndAuthMessageAPI(const std::string& apiData,
                                const std::string& xyzChallange) override {}
    void verifyAndAuthCompleted(bool isSuccessful, const std::string& errorText) override {}
    void displayGroupList(int reqId, const std::string& groups) override {}
    void displayGroupUpdated(int reqId, const std::string& contractInfo) override {}
    void positionMulti(int reqId, const std::string& account,
                      const std::string& modelCode, const Contract& contract,
                      Decimal pos, double avgCost) override {}
    void positionMultiEnd(int reqId) override {}
    void accountUpdateMulti(int reqId, const std::string& account,
                          const std::string& modelCode, const std::string& key,
                          const std::string& value, const std::string& currency) override {}
    void accountUpdateMultiEnd(int reqId) override {}
    void securityDefinitionOptionalParameter(int reqId, const std::string& exchange,
                                            int underlyingConId, const std::string& tradingClass,
                                            const std::string& multiplier, const std::set<std::string>& expirations,
                                            const std::set<double>& strikes) override {}
    void securityDefinitionOptionalParameterEnd(int reqId) override {}
    void softDollarTiers(int reqId, const std::vector<SoftDollarTier>& tiers) override {}
    void familyCodes(const std::vector<FamilyCode>& familyCodes) override {}
    void symbolSamples(int reqId, const std::vector<ContractDescription>& contractDescriptions) override {}
    void mktDepthExchanges(const std::vector<DepthMktDataDescription>& depthMktDataDescriptions) override {}
    void tickNews(int tickerId, time_t timeStamp, const std::string& providerCode,
                 const std::string& articleId, const std::string& headline,
                 const std::string& extraData) override {}
    void smartComponents(int reqId, const SmartComponentsMap& theMap) override {}
    void tickReqParams(int tickerId, double minTick, const std::string& bboExchange,
                      int snapshotPermissions) override {}
    void newsProviders(const std::vector<NewsProvider>& newsProviders) override {}
    void newsArticle(int requestId, int articleType, const std::string& articleText) override {}
    void historicalNews(int requestId, const std::string& time, const std::string& providerCode,
                       const std::string& articleId, const std::string& headline) override {}
    void historicalNewsEnd(int requestId, bool hasMore) override {}
    void headTimestamp(int reqId, const std::string& headTimestamp) override {}
    void histogramData(int reqId, const HistogramDataVector& data) override {}
    void historicalDataUpdate(TickerId reqId, const Bar& bar) override {}
    void rerouteMktDataReq(int reqId, int conid, const std::string& exchange) override {}
    void rerouteMktDepthReq(int reqId, int conid, const std::string& exchange) override {}
    void marketRule(int marketRuleId, const std::vector<PriceIncrement>& priceIncrements) override {}
    void pnl(int reqId, double dailyPnL, double unrealizedPnL, double realizedPnL) override {}
    void pnlSingle(int reqId, Decimal pos, double dailyPnL, double unrealizedPnL,
                  double realizedPnL, double value) override {}
    void historicalTicks(int reqId, const std::vector<HistoricalTick>& ticks, bool done) override {}
    void historicalTicksBidAsk(int reqId, const std::vector<HistoricalTickBidAsk>& ticks,
                              bool done) override {}
    void historicalTicksLast(int reqId, const std::vector<HistoricalTickLast>& ticks,
                           bool done) override {}
    void tickByTickAllLast(int reqId, int tickType, time_t time, double price,
                          Decimal size, const TickAttribLast& tickAttribLast,
                          const std::string& exchange, const std::string& specialConditions) override {}
    void tickByTickBidAsk(int reqId, time_t time, double bidPrice, double askPrice,
                         Decimal bidSize, Decimal askSize, const TickAttribBidAsk& tickAttribBidAsk) override {}
    void tickByTickMidPoint(int reqId, time_t time, double midPoint) override {}
    void orderBound(long long orderId, int apiClientId, int apiOrderId) override {}
    void completedOrder(const Contract& contract, const Order& order,
                       const OrderState& orderState) override {}
    void completedOrdersEnd() override {}
    void replaceFAEnd(int reqId, const std::string& text) override {}
    void wshMetaData(int reqId, const std::string& dataJson) override {}
    void wshEventData(int reqId, const std::string& dataJson) override {}
    void historicalSchedule(int reqId, const std::string& startDateTime,
                          const std::string& endDateTime, const std::string& timeZone,
                          const std::vector<HistoricalSession>& sessions) override {}
    void userInfo(int reqId, const std::string& whiteBrandingId) override {}

    // ========================================================================
    // Member Variables
    // ========================================================================

    config::TWSConfig config_;
    EReaderOSSignal signal_;
    EClientSocket client_;

    std::atomic<bool> connected_;
    std::atomic<int> next_order_id_;
    std::atomic<int> next_request_id_;
    ConnectionState state_;

    std::unique_ptr<std::thread> reader_thread_;

    // Connection synchronization
    mutable std::mutex connection_mutex_;
    std::condition_variable connection_cv_;

    // Market data
    mutable std::mutex data_mutex_;
    std::map<int, types::MarketData> market_data_;
    std::map<int, MarketDataCallback> market_data_callbacks_;

    // Orders
    mutable std::mutex order_mutex_;
    std::map<int, types::Order> orders_;
    OrderStatusCallback order_status_callback_;

    // Positions
    mutable std::mutex position_mutex_;
    std::vector<types::Position> positions_;
    PositionCallback position_callback_;

    // Account
    mutable std::mutex account_mutex_;
    types::AccountInfo account_info_;
    AccountCallback account_callback_;

    // Callbacks
    ErrorCallback error_callback_;
};

// ============================================================================
// TWSClient Public Interface (delegates to Impl)
// ============================================================================

TWSClient::TWSClient(const config::TWSConfig& config)
    : pimpl_(std::make_unique<Impl>(config)) {
    spdlog::debug("TWSClient created with real TWS API integration");
}

TWSClient::~TWSClient() {
    if (is_connected()) {
        disconnect();
    }
    spdlog::debug("TWSClient destroyed");
}

bool TWSClient::connect() {
    return pimpl_->connect();
}

void TWSClient::disconnect() {
    pimpl_->disconnect();
}

bool TWSClient::is_connected() const {
    return pimpl_->is_connected();
}

ConnectionState TWSClient::get_connection_state() const {
    return pimpl_->get_connection_state();
}

void TWSClient::process_messages(int timeout_ms) {
    pimpl_->process_messages(timeout_ms);
}

// ============================================================================
// Market Data Operations
// ============================================================================

int TWSClient::request_market_data(const types::OptionContract& contract,
                                   MarketDataCallback callback) {
    return pimpl_->request_market_data(contract, callback);
}

void TWSClient::cancel_market_data(int request_id) {
    pimpl_->cancel_market_data(request_id);
}

std::vector<types::OptionContract> TWSClient::request_option_chain(
    const std::string& symbol,
    const std::string& expiry) {
    return pimpl_->request_option_chain(symbol, expiry);
}

// ============================================================================
// Order Operations
// ============================================================================

int TWSClient::place_order(const types::OptionContract& contract,
                          types::OrderAction action,
                          int quantity,
                          double limit_price,
                          types::TimeInForce tif) {
    return pimpl_->place_order(contract, action, quantity, limit_price, tif);
}

void TWSClient::cancel_order(int order_id) {
    pimpl_->cancel_order(order_id);
}

void TWSClient::cancel_all_orders() {
    pimpl_->cancel_all_orders();
}

std::optional<types::Order> TWSClient::get_order(int order_id) const {
    return pimpl_->get_order(order_id);
}

std::vector<types::Order> TWSClient::get_active_orders() const {
    return pimpl_->get_active_orders();
}

// ============================================================================
// Position Operations
// ============================================================================

void TWSClient::request_positions(PositionCallback callback) {
    pimpl_->request_positions(callback);
}

std::vector<types::Position> TWSClient::get_positions() const {
    return pimpl_->get_positions();
}

std::optional<types::Position> TWSClient::get_position(
    const types::OptionContract& contract) const {
    return pimpl_->get_position(contract);
}

// ============================================================================
// Account Operations
// ============================================================================

void TWSClient::request_account_updates(AccountCallback callback) {
    pimpl_->request_account_updates(callback);
}

std::optional<types::AccountInfo> TWSClient::get_account_info() const {
    return pimpl_->get_account_info();
}

// ============================================================================
// Callbacks
// ============================================================================

void TWSClient::set_order_status_callback(OrderStatusCallback callback) {
    pimpl_->set_order_status_callback(callback);
}

void TWSClient::set_error_callback(ErrorCallback callback) {
    pimpl_->set_error_callback(callback);
}

// ============================================================================
// Helper Methods
// ============================================================================

int TWSClient::get_next_order_id() const {
    return pimpl_->get_next_order_id();
}

bool TWSClient::is_market_open() const {
    return pimpl_->is_market_open();
}

std::chrono::system_clock::time_point TWSClient::get_server_time() const {
    return pimpl_->get_server_time();
}

} // namespace tws

// ============================================================================
// types.h implementations
// ============================================================================

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
    // TODO: Implement proper DTE calculation
    return 30;  // Stub
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
