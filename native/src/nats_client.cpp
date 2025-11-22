// nats_client.cpp - NATS message queue client wrapper implementation
#include "nats_client.h"
#include <spdlog/spdlog.h>
#include <sstream>
#include <iomanip>
#include <ctime>
#include <uuid/uuid.h>

// NATS C client headers (only if ENABLE_NATS is defined)
#ifdef ENABLE_NATS
#include "nats.h"
#define NATS_AVAILABLE 1
#else
#define NATS_AVAILABLE 0
#endif

namespace nats {

// ============================================================================
// Implementation Details
// ============================================================================

struct NatsClientImpl {
#ifdef ENABLE_NATS
    natsConnection* conn = nullptr;
#endif
    std::string url;
    bool connected = false;

    NatsClientImpl(const std::string& server_url) : url(server_url) {}

    ~NatsClientImpl() {
        disconnect();
    }

    bool connect_impl() {
#if NATS_AVAILABLE
        natsOptions* opts = nullptr;
        natsStatus status = natsOptions_Create(&opts);
        if (status != NATS_OK) {
            spdlog::warn("Failed to create NATS options: {}", natsStatus_GetText(status));
            return false;
        }

        status = natsOptions_SetURL(opts, url.c_str());
        if (status != NATS_OK) {
            spdlog::warn("Failed to set NATS URL: {}", natsStatus_GetText(status));
            natsOptions_Destroy(opts);
            return false;
        }

        status = natsConnection_Connect(&conn, opts);
        natsOptions_Destroy(opts);

        if (status == NATS_OK) {
            connected = true;
            spdlog::info("Connected to NATS at {}", url);
            return true;
        } else {
            spdlog::warn("Failed to connect to NATS at {}: {}", url, natsStatus_GetText(status));
            connected = false;
            return false;
        }
#else
        spdlog::debug("NATS integration disabled (ENABLE_NATS not set)");
        return false;
#endif
    }

    void disconnect_impl() {
#if NATS_AVAILABLE
        if (conn) {
            natsConnection_Close(conn);
            natsConnection_Destroy(conn);
            conn = nullptr;
        }
#endif
        connected = false;
    }

    bool publish_impl(const std::string& topic, const std::string& message) {
#if NATS_AVAILABLE
        if (!connected || !conn) {
            return false;
        }

        natsStatus status = natsConnection_PublishString(conn, topic.c_str(), message.c_str());
        if (status == NATS_OK) {
            spdlog::trace("Published to NATS topic {}: {}", topic, message);
            return true;
        } else {
            spdlog::warn("Failed to publish to NATS topic {}: {}", topic, natsStatus_GetText(status));
            return false;
        }
#else
        (void)topic;
        (void)message;
        return false;
#endif
    }
};

// ============================================================================
// Public Interface
// ============================================================================

NatsClient::NatsClient(const std::string& url)
    : pimpl_(std::make_unique<NatsClientImpl>(url)) {
}

NatsClient::~NatsClient() = default;

NatsClient::NatsClient(NatsClient&&) noexcept = default;
NatsClient& NatsClient::operator=(NatsClient&&) noexcept = default;

bool NatsClient::connect() {
    return pimpl_->connect_impl();
}

void NatsClient::disconnect() {
    pimpl_->disconnect_impl();
}

bool NatsClient::is_connected() const {
    return pimpl_->connected;
}

// Helper: Generate UUID string
static std::string generate_uuid() {
    uuid_t uuid;
    char uuid_str[37];
    uuid_generate(uuid);
    uuid_unparse_lower(uuid, uuid_str);
    return std::string(uuid_str);
}

// Helper: Get current timestamp in ISO 8601 format
static std::string get_iso_timestamp() {
    auto now = std::chrono::system_clock::now();
    auto time_t = std::chrono::system_clock::to_time_t(now);
    auto ms = std::chrono::duration_cast<std::chrono::milliseconds>(
        now.time_since_epoch()
    ) % 1000;

    std::tm tm_buf;
    gmtime_r(&time_t, &tm_buf);

    std::ostringstream oss;
    oss << std::put_time(&tm_buf, "%Y-%m-%dT%H:%M:%S");
    oss << '.' << std::setfill('0') << std::setw(3) << ms.count();
    oss << "Z";
    return oss.str();
}

// Helper: Create JSON message
static std::string create_json_message(
    const std::string& type,
    const std::string& source,
    const std::string& payload_json
) {
    std::ostringstream oss;
    oss << "{"
        << "\"id\":\"" << generate_uuid() << "\","
        << "\"timestamp\":\"" << get_iso_timestamp() << "\","
        << "\"source\":\"" << source << "\","
        << "\"type\":\"" << type << "\","
        << "\"payload\":" << payload_json
        << "}";
    return oss.str();
}

bool NatsClient::publish_market_data(
    const std::string& symbol,
    double bid,
    double ask,
    const std::string& timestamp
) {
    if (!is_connected()) {
        return false;
    }

    std::ostringstream payload;
    payload << "{"
            << "\"symbol\":\"" << symbol << "\","
            << "\"bid\":" << bid << ","
            << "\"ask\":" << ask << ","
            << "\"timestamp\":\"" << timestamp << "\""
            << "}";

    std::string message = create_json_message(
        "MarketDataTick",
        "cpp-tws-client",
        payload.str()
    );

    std::string topic = "market-data.tick." + symbol;
    return pimpl_->publish_impl(topic, message);
}

bool NatsClient::publish_strategy_signal(
    const std::string& symbol,
    double price,
    const std::string& signal_type
) {
    if (!is_connected()) {
        return false;
    }

    std::ostringstream payload;
    payload << "{"
            << "\"symbol\":\"" << symbol << "\","
            << "\"price\":" << price << ","
            << "\"signal_type\":\"" << signal_type << "\","
            << "\"timestamp\":\"" << get_iso_timestamp() << "\""
            << "}";

    std::string message = create_json_message(
        "StrategySignal",
        "cpp-tws-client",
        payload.str()
    );

    std::string topic = "strategy.signal." + symbol;
    return pimpl_->publish_impl(topic, message);
}

bool NatsClient::publish_strategy_decision(
    const std::string& symbol,
    int quantity,
    const std::string& side,
    double mark,
    const std::string& decision_type
) {
    if (!is_connected()) {
        return false;
    }

    std::ostringstream payload;
    payload << "{"
            << "\"symbol\":\"" << symbol << "\","
            << "\"quantity\":" << quantity << ","
            << "\"side\":\"" << side << "\","
            << "\"mark\":" << mark << ","
            << "\"decision_type\":\"" << decision_type << "\","
            << "\"timestamp\":\"" << get_iso_timestamp() << "\""
            << "}";

    std::string message = create_json_message(
        "StrategyDecision",
        "cpp-tws-client",
        payload.str()
    );

    std::string topic = "strategy.decision." + symbol;
    return pimpl_->publish_impl(topic, message);
}

} // namespace nats
