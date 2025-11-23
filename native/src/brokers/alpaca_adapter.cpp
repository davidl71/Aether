// alpaca_adapter.cpp - Alpaca adapter implementation
#include "brokers/alpaca_adapter.h"
#include "brokers/http_client.h"
#include <spdlog/spdlog.h>
#include <nlohmann/json.hpp>
#include <sstream>
#include <iomanip>
#include <chrono>
#include <thread>
#include <queue>

using json = nlohmann::json;

namespace brokers {

// ============================================================================
// Rate Limiter Implementation
// ============================================================================

class RateLimiter {
public:
    RateLimiter(int requests_per_minute = 200) : requests_per_minute_(requests_per_minute) {}

    void wait_if_needed() {
        auto now = std::chrono::steady_clock::now();

        // Remove requests older than 1 minute
        while (!request_times_.empty() &&
               (now - request_times_.front()) > std::chrono::minutes(1)) {
            request_times_.pop();
        }

        // Wait if at limit
        if (request_times_.size() >= static_cast<size_t>(requests_per_minute_)) {
            auto wait_time = std::chrono::minutes(1) - (now - request_times_.front());
            if (wait_time.count() > 0) {
                std::this_thread::sleep_for(wait_time);
            }
        }

        request_times_.push(std::chrono::steady_clock::now());
    }

private:
    int requests_per_minute_;
    std::queue<std::chrono::steady_clock::time_point> request_times_;
};

// ============================================================================
// Constructor / Destructor
// ============================================================================

AlpacaAdapter::AlpacaAdapter(const Config& config)
    : config_(config)
    , http_client_(std::make_unique<HttpClient>())
    , connection_state_(ConnectionState::Disconnected)
    , next_request_id_(1)
    , polling_active_(false)
{
    spdlog::info("AlpacaAdapter created (paper_trading={})", config_.paper_trading);
}

AlpacaAdapter::~AlpacaAdapter() {
    disconnect();
    if (polling_thread_.joinable()) {
        polling_active_ = false;
        polling_thread_.join();
    }
}

// ============================================================================
// Symbol Conversion Helpers
// ============================================================================

static std::string convert_to_alpaca_symbol(const box_spread::types::OptionContract& contract) {
    // Format: SYMBOL + YYYYMMDD + C/P + STRIKE*1000 (zero-padded to 8 digits)
    std::string symbol = contract.symbol;
    std::string expiry = contract.expiry;  // Already in YYYYMMDD format
    char right = (contract.type == box_spread::types::OptionType::Call) ? 'C' : 'P';

    // Strike in cents (multiply by 1000, then format as 8-digit integer)
    int strike_cents = static_cast<int>(contract.strike * 1000);

    std::ostringstream oss;
    oss << symbol << expiry << right << std::setfill('0') << std::setw(8) << strike_cents;
    return oss.str();
}

static box_spread::types::OptionContract parse_alpaca_symbol(const std::string& alpaca_symbol) {
    // Parse: SPY240119C00450000 -> symbol=SPY, expiry=240119, type=Call, strike=450.0
    box_spread::types::OptionContract contract;

    // Find the right character (C or P)
    size_t right_pos = alpaca_symbol.find_last_of("CP");
    if (right_pos == std::string::npos || right_pos < 3) {
        return contract;  // Invalid format
    }

    // Extract symbol (everything before expiry)
    size_t expiry_start = right_pos - 6;  // YYYYMMDD is 6 digits before C/P
    if (expiry_start == 0) {
        return contract;  // Invalid format
    }

    contract.symbol = alpaca_symbol.substr(0, expiry_start);
    contract.expiry = alpaca_symbol.substr(expiry_start, 6);
    contract.type = (alpaca_symbol[right_pos] == 'C')
        ? box_spread::types::OptionType::Call
        : box_spread::types::OptionType::Put;

    // Extract strike (8 digits after C/P)
    if (right_pos + 9 <= alpaca_symbol.length()) {
        std::string strike_str = alpaca_symbol.substr(right_pos + 1, 8);
        int strike_cents = std::stoi(strike_str);
        contract.strike = strike_cents / 1000.0;
    }

    return contract;
}

// ============================================================================
// Type Conversion Helpers
// ============================================================================

box_spread::types::MarketData AlpacaAdapter::convert_market_data(
    const std::string& symbol, double bid, double ask, double last
) {
    box_spread::types::MarketData data;
    data.symbol = symbol;
    data.timestamp = std::chrono::system_clock::now();
    data.bid = bid;
    data.ask = ask;
    data.last = last;
    data.bid_size = 0;  // Alpaca may not provide size
    data.ask_size = 0;
    data.last_size = 0;
    return data;
}

box_spread::types::Order AlpacaAdapter::convert_order(
    const std::string& order_id, const std::string& symbol, const std::string& status
) {
    box_spread::types::Order order;
    order.order_id = std::stoi(order_id);
    // Contract parsing would be needed for full order
    order.status = (status == "filled")
        ? box_spread::types::OrderStatus::Filled
        : box_spread::types::OrderStatus::Pending;
    order.submitted_time = std::chrono::system_clock::now();
    order.last_update = std::chrono::system_clock::now();
    return order;
}

box_spread::types::Position AlpacaAdapter::convert_position(
    const std::string& symbol, int qty, double avg_price
) {
    box_spread::types::Position pos;
    // Contract parsing would be needed for full position
    pos.quantity = qty;
    pos.avg_price = avg_price;
    pos.timestamp = std::chrono::system_clock::now();
    return pos;
}

// ============================================================================
// HTTP Client Helpers
// ============================================================================

std::string AlpacaAdapter::get_auth_headers() const {
    // Return JSON string for headers map
    json headers;
    headers["APCA-API-KEY-ID"] = config_.api_key_id;
    headers["APCA-API-SECRET-KEY"] = config_.api_secret_key;
    return headers.dump();
}

std::string AlpacaAdapter::make_request(
    const std::string& method,
    const std::string& endpoint,
    const std::string& body
) {
    std::string url = config_.base_url + endpoint;

    std::map<std::string, std::string> headers;
    headers["APCA-API-KEY-ID"] = config_.api_key_id;
    headers["APCA-API-SECRET-KEY"] = config_.api_secret_key;
    headers["Content-Type"] = "application/json";

    // Rate limiting
    static RateLimiter rate_limiter(200);  // 200 requests per minute
    rate_limiter.wait_if_needed();

    std::string response;
    if (method == "GET") {
        response = http_client_->get(url, headers);
    } else if (method == "POST") {
        response = http_client_->post(url, body, headers);
    } else if (method == "PUT") {
        response = http_client_->put(url, body, headers);
    } else if (method == "DELETE") {
        response = http_client_->del(url, headers);
    }

    // Check for errors
    long status_code = http_client_->get_last_response_code();
    if (status_code >= 400) {
        spdlog::error("Alpaca API error {}: {}", status_code, response);
        if (error_callback_) {
            error_callback_(static_cast<int>(status_code), response);
        }
    }

    return response;
}

// ============================================================================
// Connection Management
// ============================================================================

bool AlpacaAdapter::connect() {
    std::lock_guard<std::mutex> lock(mutex_);

    if (connected_.load()) {
        return true;
    }

    connection_state_ = ConnectionState::Connecting;

    // Test connection by getting account info
    try {
        std::string response = make_request("GET", "/v2/account", "");
        if (!response.empty()) {
            json account = json::parse(response);
            if (account.contains("account_number")) {
                connected_ = true;
                connection_state_ = ConnectionState::Connected;
                spdlog::info("AlpacaAdapter connected successfully");
                return true;
            }
        }
    } catch (const std::exception& e) {
        spdlog::error("AlpacaAdapter connection failed: {}", e.what());
    }

    connection_state_ = ConnectionState::Error;
    connected_ = false;
    return false;
}

void AlpacaAdapter::disconnect() {
    std::lock_guard<std::mutex> lock(mutex_);
    stop_market_data_polling();
    connected_ = false;
    connection_state_ = ConnectionState::Disconnected;
    subscriptions_.clear();
    callbacks_.clear();
    spdlog::info("AlpacaAdapter disconnected");
}

bool AlpacaAdapter::is_connected() const {
    return connected_.load();
}

box_spread::brokers::ConnectionState AlpacaAdapter::get_connection_state() const {
    return connection_state_.load();
}

box_spread::brokers::BrokerType AlpacaAdapter::get_broker_type() const {
    return box_spread::brokers::BrokerType::ALPACA;
}

box_spread::brokers::BrokerCapabilities AlpacaAdapter::get_capabilities() const {
    box_spread::brokers::BrokerCapabilities caps;
    caps.supports_options = true;
    caps.supports_multi_leg_orders = true;
    caps.supports_real_time_data = false;  // Using polling, not WebSocket
    caps.supports_historical_data = true;
    caps.max_orders_per_second = 10;  // Conservative limit
    caps.rate_limit_per_minute = 200;  // Alpaca limit
    return caps;
}

// ============================================================================
// Market Data
// ============================================================================

int AlpacaAdapter::request_market_data(
    const box_spread::types::OptionContract& contract,
    std::function<void(const box_spread::types::MarketData&)> callback
) {
    std::lock_guard<std::mutex> lock(mutex_);

    if (!connected_.load()) {
        spdlog::error("AlpacaAdapter not connected");
        return -1;
    }

    int request_id = next_request_id_++;
    subscriptions_[request_id] = contract;
    callbacks_[request_id] = callback;

    // Start polling if not already active
    if (!polling_active_.load()) {
        start_market_data_polling();
    }

    return request_id;
}

void AlpacaAdapter::cancel_market_data(int request_id) {
    std::lock_guard<std::mutex> lock(mutex_);
    subscriptions_.erase(request_id);
    callbacks_.erase(request_id);

    // Stop polling if no subscriptions
    if (subscriptions_.empty()) {
        stop_market_data_polling();
    }
}

std::optional<box_spread::types::MarketData> AlpacaAdapter::request_market_data_sync(
    const box_spread::types::OptionContract& contract,
    int timeout_ms
) {
    std::string alpaca_symbol = convert_to_alpaca_symbol(contract);

    // Request latest quote
    std::string endpoint = "/v2/options/quotes/latest?symbols=" + alpaca_symbol;
    std::string response = make_request("GET", endpoint, "");

    if (response.empty()) {
        return std::nullopt;
    }

    try {
        json data = json::parse(response);
        if (data.contains("quotes") && data["quotes"].is_array() && !data["quotes"].empty()) {
            json quote = data["quotes"][0];
            double bid = quote.value("bp", 0.0);
            double ask = quote.value("ap", 0.0);
            double last = quote.value("p", (bid + ask) / 2.0);

            return convert_market_data(alpaca_symbol, bid, ask, last);
        }
    } catch (const std::exception& e) {
        spdlog::error("Failed to parse market data response: {}", e.what());
    }

    return std::nullopt;
}

// ============================================================================
// Options Chain
// ============================================================================

std::vector<box_spread::types::OptionContract> AlpacaAdapter::request_option_chain(
    const std::string& symbol,
    const std::string& expiry
) {
    std::lock_guard<std::mutex> lock(mutex_);

    std::vector<box_spread::types::OptionContract> contracts;

    try {
        std::string endpoint = "/v2/options/contracts?underlying_symbol=" + symbol;
        if (!expiry.empty()) {
            endpoint += "&expiration_date=" + expiry;
        }

        std::string response = make_request("GET", endpoint, "");
        if (response.empty()) {
            return contracts;
        }

        json data = json::parse(response);
        if (data.contains("contracts") && data["contracts"].is_array()) {
            for (const auto& contract_json : data["contracts"]) {
                box_spread::types::OptionContract contract;
                contract.symbol = symbol;
                contract.expiry = contract_json.value("expiration_date", "");
                contract.strike = contract_json.value("strike_price", 0.0);
                contract.type = (contract_json.value("type", "") == "call")
                    ? box_spread::types::OptionType::Call
                    : box_spread::types::OptionType::Put;
                contract.style = box_spread::types::OptionStyle::American;  // Alpaca default
                contract.exchange = contract_json.value("exchange", "");
                contract.local_symbol = contract_json.value("symbol", "");

                contracts.push_back(contract);
            }
        }
    } catch (const std::exception& e) {
        spdlog::error("Failed to request option chain: {}", e.what());
    }

    return contracts;
}

// ============================================================================
// Contract Details
// ============================================================================

int AlpacaAdapter::request_contract_details(
    const box_spread::types::OptionContract& contract,
    std::function<void(long conId)> callback
) {
    // Alpaca doesn't use contract IDs like IB, so we'll use a hash of the symbol
    std::string alpaca_symbol = convert_to_alpaca_symbol(contract);
    long contract_id = std::hash<std::string>{}(alpaca_symbol);

    // Cache the contract ID
    contract_id_cache_[alpaca_symbol] = contract_id;

    // Call callback immediately (Alpaca doesn't require async lookup)
    callback(contract_id);

    return 1;  // Request ID
}

long AlpacaAdapter::request_contract_details_sync(
    const box_spread::types::OptionContract& contract,
    int timeout_ms
) {
    std::string alpaca_symbol = convert_to_alpaca_symbol(contract);

    // Check cache first
    if (contract_id_cache_.find(alpaca_symbol) != contract_id_cache_.end()) {
        return contract_id_cache_[alpaca_symbol];
    }

    // Generate contract ID from symbol hash
    long contract_id = std::hash<std::string>{}(alpaca_symbol);
    contract_id_cache_[alpaca_symbol] = contract_id;

    return contract_id;
}

long AlpacaAdapter::lookup_contract_id(const box_spread::types::OptionContract& contract) {
    std::string alpaca_symbol = convert_to_alpaca_symbol(contract);

    if (contract_id_cache_.find(alpaca_symbol) != contract_id_cache_.end()) {
        return contract_id_cache_[alpaca_symbol];
    }

    return request_contract_details_sync(contract);
}

// ============================================================================
// Order Management
// ============================================================================

int AlpacaAdapter::place_order(
    const box_spread::types::OptionContract& contract,
    box_spread::types::OrderAction action,
    int quantity,
    double limit_price,
    box_spread::types::TimeInForce tif
) {
    std::lock_guard<std::mutex> lock(mutex_);

    if (!connected_.load()) {
        spdlog::error("AlpacaAdapter not connected");
        return -1;
    }

    std::string alpaca_symbol = convert_to_alpaca_symbol(contract);

    json order;
    order["symbol"] = alpaca_symbol;
    order["qty"] = quantity;
    order["side"] = (action == box_spread::types::OrderAction::Buy) ? "buy" : "sell";
    order["type"] = (limit_price > 0.0) ? "limit" : "market";
    if (limit_price > 0.0) {
        order["limit_price"] = limit_price;
    }
    order["time_in_force"] = (tif == box_spread::types::TimeInForce::Day) ? "day" : "gtc";

    std::string body = order.dump();
    std::string response = make_request("POST", "/v2/orders", body);

    if (response.empty()) {
        return -1;
    }

    try {
        json order_response = json::parse(response);
        if (order_response.contains("id")) {
            std::string order_id_str = order_response["id"];
            return std::stoi(order_id_str);
        }
    } catch (const std::exception& e) {
        spdlog::error("Failed to parse order response: {}", e.what());
    }

    return -1;
}

bool AlpacaAdapter::cancel_order(int order_id) {
    std::lock_guard<std::mutex> lock(mutex_);

    if (!connected_.load()) {
        return false;
    }

    std::string endpoint = "/v2/orders/" + std::to_string(order_id);
    std::string response = make_request("DELETE", endpoint, "");

    return !response.empty() && http_client_->get_last_response_code() == 204;
}

std::optional<box_spread::types::Order> AlpacaAdapter::get_order_status(int order_id) const {
    std::lock_guard<std::mutex> lock(mutex_);

    if (!connected_.load()) {
        return std::nullopt;
    }

    std::string endpoint = "/v2/orders/" + std::to_string(order_id);
    std::string response = const_cast<AlpacaAdapter*>(this)->make_request("GET", endpoint, "");

    if (response.empty()) {
        return std::nullopt;
    }

    try {
        json order_json = json::parse(response);
        box_spread::types::Order order;
        order.order_id = order_id;
        order.quantity = order_json.value("qty", 0);
        order.filled_quantity = order_json.value("filled_qty", 0);
        order.avg_fill_price = order_json.value("filled_avg_price", 0.0);

        std::string status = order_json.value("status", "");
        if (status == "filled") {
            order.status = box_spread::types::OrderStatus::Filled;
        } else if (status == "partially_filled") {
            order.status = box_spread::types::OrderStatus::PartiallyFilled;
        } else if (status == "canceled") {
            order.status = box_spread::types::OrderStatus::Cancelled;
        } else {
            order.status = box_spread::types::OrderStatus::Pending;
        }

        return order;
    } catch (const std::exception& e) {
        spdlog::error("Failed to parse order status: {}", e.what());
    }

    return std::nullopt;
}

// ============================================================================
// Multi-Leg Orders (Box Spreads)
// ============================================================================

int AlpacaAdapter::place_combo_order(
    const std::vector<box_spread::types::OptionContract>& contracts,
    const std::vector<box_spread::types::OrderAction>& actions,
    const std::vector<int>& quantities,
    const std::vector<long>& contract_ids,
    const std::vector<double>& limit_prices
) {
    std::lock_guard<std::mutex> lock(mutex_);

    if (!connected_.load()) {
        spdlog::error("AlpacaAdapter not connected");
        return -1;
    }

    if (contracts.size() != actions.size() || contracts.size() != quantities.size()) {
        spdlog::error("Mismatched combo order parameters");
        return -1;
    }

    // Alpaca multi-leg orders use "legs" array
    json order;
    order["class"] = "bracket";  // or "oco", "oto" depending on strategy
    order["symbol"] = contracts[0].symbol;  // Underlying symbol
    order["legs"] = json::array();

    for (size_t i = 0; i < contracts.size(); ++i) {
        json leg;
        leg["symbol"] = convert_to_alpaca_symbol(contracts[i]);
        leg["qty"] = quantities[i];
        leg["side"] = (actions[i] == box_spread::types::OrderAction::Buy) ? "buy" : "sell";
        leg["type"] = (limit_prices[i] > 0.0) ? "limit" : "market";
        if (limit_prices[i] > 0.0) {
            leg["limit_price"] = limit_prices[i];
        }
        order["legs"].push_back(leg);
    }

    std::string body = order.dump();
    std::string response = make_request("POST", "/v2/orders", body);

    if (response.empty()) {
        return -1;
    }

    try {
        json order_response = json::parse(response);
        if (order_response.contains("id")) {
            std::string order_id_str = order_response["id"];
            return std::stoi(order_id_str);
        }
    } catch (const std::exception& e) {
        spdlog::error("Failed to parse combo order response: {}", e.what());
    }

    return -1;
}

// ============================================================================
// Positions
// ============================================================================

std::vector<box_spread::types::Position> AlpacaAdapter::get_positions() {
    std::lock_guard<std::mutex> lock(mutex_);

    std::vector<box_spread::types::Position> positions;

    if (!connected_.load()) {
        return positions;
    }

    try {
        std::string response = make_request("GET", "/v2/positions", "");
        if (response.empty()) {
            return positions;
        }

        json data = json::parse(response);
        if (data.is_array()) {
            for (const auto& pos_json : data) {
                box_spread::types::Position pos;
                std::string symbol = pos_json.value("symbol", "");
                pos.quantity = pos_json.value("qty", 0);
                pos.avg_price = pos_json.value("avg_entry_price", 0.0);
                pos.timestamp = std::chrono::system_clock::now();

                // Try to parse contract from symbol
                pos.contract = parse_alpaca_symbol(symbol);

                positions.push_back(pos);
            }
        }
    } catch (const std::exception& e) {
        spdlog::error("Failed to get positions: {}", e.what());
    }

    return positions;
}

std::optional<box_spread::types::Position> AlpacaAdapter::get_position(
    const box_spread::types::OptionContract& contract
) {
    auto positions = get_positions();
    std::string target_symbol = convert_to_alpaca_symbol(contract);

    for (const auto& pos : positions) {
        std::string pos_symbol = convert_to_alpaca_symbol(pos.contract);
        if (pos_symbol == target_symbol) {
            return pos;
        }
    }

    return std::nullopt;
}

// ============================================================================
// Account Information
// ============================================================================

std::optional<box_spread::types::AccountInfo> AlpacaAdapter::get_account_info() {
    std::lock_guard<std::mutex> lock(mutex_);

    if (!connected_.load()) {
        return std::nullopt;
    }

    try {
        std::string response = make_request("GET", "/v2/account", "");
        if (response.empty()) {
            return std::nullopt;
        }

        json account = json::parse(response);
        box_spread::types::AccountInfo info;
        info.account_id = account.value("account_number", "");
        info.net_liquidation = account.value("equity", 0.0);
        info.cash_balance = account.value("cash", 0.0);
        info.buying_power = account.value("buying_power", 0.0);
        info.maintenance_margin = account.value("maintenance_margin", 0.0);
        info.initial_margin = account.value("initial_margin", 0.0);
        info.unrealized_pnl = account.value("unrealized_pl", 0.0);
        info.realized_pnl = 0.0;  // Alpaca may not provide this directly
        info.timestamp = std::chrono::system_clock::now();
        info.last_update = info.timestamp;

        return info;
    } catch (const std::exception& e) {
        spdlog::error("Failed to get account info: {}", e.what());
    }

    return std::nullopt;
}

double AlpacaAdapter::get_buying_power() {
    auto account = get_account_info();
    return account ? account->buying_power : 0.0;
}

double AlpacaAdapter::get_net_liquidation_value() {
    auto account = get_account_info();
    return account ? account->net_liquidation : 0.0;
}

// ============================================================================
// Error Handling
// ============================================================================

void AlpacaAdapter::set_error_callback(
    std::function<void(int code, const std::string& msg)> callback
) {
    std::lock_guard<std::mutex> lock(mutex_);
    error_callback_ = callback;
}

// ============================================================================
// Market Data Polling
// ============================================================================

void AlpacaAdapter::start_market_data_polling() {
    if (polling_active_.exchange(true)) {
        return;  // Already polling
    }

    polling_thread_ = std::thread(&AlpacaAdapter::poll_market_data_loop, this);
    spdlog::info("AlpacaAdapter market data polling started");
}

void AlpacaAdapter::stop_market_data_polling() {
    if (!polling_active_.exchange(false)) {
        return;  // Not polling
    }

    if (polling_thread_.joinable()) {
        polling_thread_.join();
    }

    spdlog::info("AlpacaAdapter market data polling stopped");
}

void AlpacaAdapter::poll_market_data_loop() {
    while (polling_active_.load()) {
        std::this_thread::sleep_for(std::chrono::milliseconds(config_.poll_interval_ms));

        std::lock_guard<std::mutex> lock(mutex_);

        if (subscriptions_.empty()) {
            continue;
        }

        // Poll each subscription
        for (const auto& [request_id, contract] : subscriptions_) {
            auto data = request_market_data_sync(contract, 5000);
            if (data && callbacks_.find(request_id) != callbacks_.end()) {
                callbacks_[request_id](*data);
            }
        }
    }
}

} // namespace brokers
