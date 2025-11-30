# NATS Integration Guide - C++

**Date**: 2025-11-20
**Status**: Planning/Implementation Guide
**Library**: `nats.c` (official NATS C client)

## Overview

This guide covers integrating NATS message queue into the C++ TWS client for publishing market data and subscribing to strategy signals.

## Prerequisites

### Installation

```bash

# Install nats.c library

git clone https://github.com/nats-io/nats.c.git
cd nats.c
mkdir build && cd build
cmake ..
make
sudo make install
```

### CMake Integration

```cmake

# Find NATS library

find_library(NATS_LIB nats REQUIRED)
find_path(NATS_INCLUDE_DIR nats/nats.h REQUIRED)

target_link_libraries(tws_client ${NATS_LIB})
target_include_directories(tws_client PRIVATE ${NATS_INCLUDE_DIR})
```

## Connection Management

### Connect to NATS Server

```cpp

#include <nats/nats.h>

natsConnection* conn = nullptr;
natsStatus status;

// Connect to NATS server
status = natsConnection_ConnectTo(&conn, "nats://localhost:4222");
if (status != NATS_OK) {
    std::cerr << "Failed to connect to NATS: " << natsStatus_GetText(status) << std::endl;
    return -1;
}

// Connection options with reconnection
natsOptions* opts = nullptr;
natsOptions_Create(&opts);
natsOptions_SetURL(opts, "nats://localhost:4222");
natsOptions_SetReconnectWait(opts, 2000);  // 2 seconds
natsOptions_SetMaxReconnects(opts, -1);    // Unlimited reconnects
natsConnection_Connect(&conn, opts);
```

## Publishing Messages

### Market Data Publishing

```cpp

#include <nlohmann/json.hpp>

// Topic: market-data.tick.{symbol}
void publish_market_data_tick(natsConnection* conn,
                               const std::string& symbol,
                               double bid, double ask) {
    // Create message payload
    nlohmann::json payload = {
        {"symbol", symbol},
        {"bid", bid},
        {"ask", ask},
        {"timestamp", get_current_timestamp()}
    };

    // Wrap in NATS message format
    nlohmann::json message = {
        {"id", generate_uuid()},
        {"timestamp", get_current_timestamp()},
        {"source", "tws-client"},
        {"type", "MarketDataTick"},
        {"payload", payload}
    };

    std::string topic = "market-data.tick." + symbol;
    std::string json_str = message.dump();

    natsStatus status = natsConnection_PublishString(
        conn,
        topic.c_str(),
        json_str.c_str()
    );

    if (status != NATS_OK) {
        std::cerr << "Failed to publish: " << natsStatus_GetText(status) << std::endl;
    }
}
```

### Error Handling with DLQ

```cpp
// Retry logic with exponential backoff
bool publish_with_retry(natsConnection* conn,
                        const std::string& topic,
                        const std::string& message,
                        int max_retries = 3) {
    int delay_ms = 100;

    for (int attempt = 0; attempt <= max_retries; ++attempt) {
        natsStatus status = natsConnection_PublishString(
            conn, topic.c_str(), message.c_str()
        );

        if (status == NATS_OK) {
            return true;
        }

        if (attempt < max_retries) {
            std::this_thread::sleep_for(std::chrono::milliseconds(delay_ms));
            delay_ms *= 2;  // Exponential backoff
        }
    }

    // Send to DLQ after all retries failed
    send_to_dlq(conn, topic, message, "publish_error", max_retries);
    return false;
}
```

## Subscribing to Messages

### Subscribe to Strategy Signals

```cpp
// Topic: strategy.signal.>
void subscribe_strategy_signals(natsConnection* conn) {
    natsSubscription* sub = nullptr;

    natsStatus status = natsConnection_Subscribe(
        &sub,
        conn,
        "strategy.signal.>",
        on_strategy_signal,
        nullptr  // closure
    );

    if (status != NATS_OK) {
        std::cerr << "Failed to subscribe: " << natsStatus_GetText(status) << std::endl;
    }
}

// Message handler
void on_strategy_signal(natsConnection* nc,
                        natsSubscription* sub,
                        natsMsg* msg,
                        void* closure) {
    const char* data = natsMsg_GetData(msg);

    try {
        nlohmann::json message = nlohmann::json::parse(data);

        if (message["type"] == "StrategySignal") {
            auto payload = message["payload"];
            std::string symbol = payload["symbol"];
            double price = payload["price"];

            // Process strategy signal
            process_strategy_signal(symbol, price);
        }
    } catch (const std::exception& e) {
        std::cerr << "Error parsing message: " << e.what() << std::endl;
    }

    natsMsg_Destroy(msg);
}
```

## Topic Constants

### Header File: `nats_topics.h`

```cpp

#ifndef NATS_TOPICS_H
#define NATS_TOPICS_H

namespace nats_topics {
    namespace market_data {
        inline std::string tick(const std::string& symbol) {
            return "market-data.tick." + symbol;
        }
        inline std::string candle(const std::string& symbol) {
            return "market-data.candle." + symbol;
        }
        inline std::string quote(const std::string& symbol) {
            return "market-data.quote." + symbol;
        }
        inline const char* all() { return "market-data.>"; }
    }

    namespace strategy {
        inline std::string signal(const std::string& symbol) {
            return "strategy.signal." + symbol;
        }
        inline std::string decision(const std::string& symbol) {
            return "strategy.decision." + symbol;
        }
        inline const char* all_signals() { return "strategy.signal.>"; }
        inline const char* all_decisions() { return "strategy.decision.>"; }
    }

    namespace dlq {
        inline std::string dead_letter(const std::string& component,
                                       const std::string& error_type) {
            return "system.dlq." + component + "." + error_type;
        }
    }
}

#endif
```

## Integration Points

### TWS Client Integration

```cpp
// In tws_client.cpp tickPrice callback
void TWSClient::tickPrice(TickerId tickerId, TickType field, double price) {
    // ... existing code ...

    // Publish to NATS
    if (nats_conn_) {
        std::string symbol = get_symbol(tickerId);
        publish_market_data_tick(nats_conn_, symbol, bid, ask);
    }
}
```

## Message Serialization

### JSON Library

Use `nlohmann/json` for JSON serialization:

```cpp

#include <nlohmann/json.hpp>

// Serialize message
nlohmann::json message = {
    {"id", generate_uuid()},
    {"timestamp", get_iso_timestamp()},
    {"source", "tws-client"},
    {"type", "MarketDataTick"},
    {"payload", {
        {"symbol", "SPY"},
        {"bid", 509.15},
        {"ask", 509.18}
    }}
};

std::string json_str = message.dump();
```

## Error Handling

### Connection Errors

```cpp
// Handle connection failures gracefully
if (natsConnection_Status(conn) != NATS_CONN_STATUS_CONNECTED) {
    // Attempt reconnection
    natsConnection_Reconnect(conn);
}
```

### Publish Errors

```cpp
natsStatus status = natsConnection_PublishString(conn, topic, message);
if (status != NATS_OK) {
    // Log error
    log_error("NATS publish failed: %s", natsStatus_GetText(status));

    // Send to DLQ if configured
    if (dlq_enabled_) {
        send_to_dlq(conn, topic, message, "publish_error", 0);
    }
}
```

## Testing

### Unit Tests

```cpp

#include <gtest/gtest.h>

TEST(NatsIntegration, PublishMarketData) {
    natsConnection* conn = nullptr;
    natsConnection_ConnectTo(&conn, "nats://localhost:4222");

    std::string topic = "market-data.tick.SPY";
    std::string message = R"({"symbol":"SPY","bid":509.15,"ask":509.18})";

    natsStatus status = natsConnection_PublishString(conn, topic.c_str(), message.c_str());
    ASSERT_EQ(status, NATS_OK);

    natsConnection_Destroy(conn);
}
```

## Configuration

### Environment Variables

```cpp
// Read NATS URL from environment
const char* nats_url = std::getenv("NATS_URL");
if (!nats_url) {
    nats_url = "nats://localhost:4222";  // Default
}

// Enable/disable NATS
bool nats_enabled = std::getenv("NATS_ENABLED") != nullptr;
```

## Best Practices

1. **Connection Reuse**: Create one connection and reuse it
2. **Error Handling**: Always check return status codes
3. **Message Validation**: Validate JSON before publishing
4. **Topic Naming**: Use topic constants from `nats_topics.h`
5. **DLQ Integration**: Implement retry logic and DLQ publishing
6. **Thread Safety**: NATS C client is thread-safe for publishing

## References

- [NATS C Client Documentation](https://github.com/nats-io/nats.c)
- [NATS Topics Registry](research/../NATS_TOPICS_REGISTRY.md)
- [Message Schemas](research/../message_schemas/README.md)
