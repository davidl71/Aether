// nats_client.h - NATS message queue client wrapper for C++ TWS client
#pragma once

#include <string>
#include <memory>
#include <optional>
#include <functional>
#include <chrono>

namespace nats {

// Forward declaration
struct NatsClientImpl;

/**
 * NATS client wrapper for publishing market data and strategy events.
 *
 * Thread-safe wrapper around nats.c library for publishing messages to NATS topics.
 * Supports graceful degradation when NATS is not available or disabled.
 */
class NatsClient {
public:
    /**
     * Constructor
     * @param url NATS server URL (e.g., "nats://127.0.0.1:4222")
     */
    explicit NatsClient(const std::string& url = "nats://127.0.0.1:4222");

    /**
     * Destructor - automatically disconnects
     */
    ~NatsClient();

    // Disable copy (manage connection state)
    NatsClient(const NatsClient&) = delete;
    NatsClient& operator=(const NatsClient&) = delete;

    // Allow move
    NatsClient(NatsClient&&) noexcept;
    NatsClient& operator=(NatsClient&&) noexcept;

    /**
     * Connect to NATS server
     * @return true if connected, false otherwise
     */
    [[nodiscard]] bool connect();

    /**
     * Disconnect from NATS server
     */
    void disconnect();

    /**
     * Check if connected to NATS
     * @return true if connected, false otherwise
     */
    [[nodiscard]] bool is_connected() const;

    /**
     * Publish market data tick to NATS
     * @param symbol Symbol (e.g., "SPX")
     * @param bid Bid price
     * @param ask Ask price
     * @param timestamp Timestamp (ISO 8601 string)
     * @return true if published, false otherwise
     */
    bool publish_market_data(
        const std::string& symbol,
        double bid,
        double ask,
        const std::string& timestamp
    );

    /**
     * Publish strategy signal to NATS
     * @param symbol Symbol
     * @param price Current price
     * @param signal_type Signal type (e.g., "opportunity", "alert")
     * @return true if published, false otherwise
     */
    bool publish_strategy_signal(
        const std::string& symbol,
        double price,
        const std::string& signal_type = "opportunity"
    );

    /**
     * Publish strategy decision to NATS
     * @param symbol Symbol
     * @param quantity Trade quantity
     * @param side Trade side ("BUY" or "SELL")
     * @param mark Mark price
     * @param decision_type Decision type (e.g., "trade", "cancel")
     * @return true if published, false otherwise
     */
    bool publish_strategy_decision(
        const std::string& symbol,
        int quantity,
        const std::string& side,
        double mark,
        const std::string& decision_type = "trade"
    );

private:
    std::unique_ptr<NatsClientImpl> pimpl_;
};

} // namespace nats
