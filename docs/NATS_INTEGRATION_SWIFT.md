# NATS Integration Guide - Swift

**Date**: 2025-11-20
**Status**: Planning/Implementation Guide
**Library**: `SwiftNATS` or `Nats.swift`

## Overview

This guide covers integrating NATS message queue into Swift iPad applications for real-time updates and publishing user actions.

## Prerequisites

### Installation

#### Swift Package Manager

```swift
// Package.swift
dependencies: [
    .package(url: "https://github.com/nats-io/swift-nats.git", from: "0.1.0")
]
```

#### CocoaPods

```ruby
# Podfile
pod 'SwiftNATS', '~> 0.1.0'
```

## Connection Management

### Connect to NATS Server

```swift
import SwiftNATS

class NATSManager {
    private var nats: NATS?

    func connect(url: String = "nats://localhost:4222") async throws {
        let options = NATSOptions()
        options.url = url
        options.reconnect = true
        options.reconnectTimeWait = 2  // 2 seconds
        options.maxReconnectAttempts = -1  // Unlimited

        nats = try await NATS.connect(options: options)
        print("Connected to NATS at \(url)")
    }

    func disconnect() {
        nats?.close()
        nats = nil
    }
}
```

## Publishing Messages

### Publish Strategy Signal

```swift
import Foundation

struct StrategySignalPayload: Codable {
    let symbol: String
    let price: Double
    let timestamp: String
}

struct NATSMessage<T: Codable>: Codable {
    let id: String
    let timestamp: String
    let source: String
    let type: String
    let payload: T
}

func publishStrategySignal(
    nats: NATS,
    symbol: String,
    price: Double
) async throws {
    let topic = "strategy.signal.\(symbol)"

    let payload = StrategySignalPayload(
        symbol: symbol,
        price: price,
        timestamp: ISO8601DateFormatter().string(from: Date())
    )

    let message = NATSMessage(
        id: UUID().uuidString,
        timestamp: ISO8601DateFormatter().string(from: Date()),
        source: "swift-ipad",
        type: "StrategySignal",
        payload: payload
    )

    let encoder = JSONEncoder()
    encoder.dateEncodingStrategy = .iso8601
    let data = try encoder.encode(message)

    do {
        try await nats.publish(topic, payload: data)
        print("Published signal for \(symbol) at \(price)")
    } catch {
        print("Failed to publish signal: \(error)")
        try await sendToDLQ(nats: nats, topic: topic, message: message, errorType: "publish_error")
    }
}
```

## Subscribing to Messages

### Subscribe to Market Data

```swift
struct MarketDataTickPayload: Codable {
    let symbol: String
    let price: Double
    let bid: Double?
    let ask: Double?
    let spread: Double?
}

func subscribeMarketData(
    nats: NATS,
    symbol: String,
    callback: @escaping (MarketDataTickPayload) -> Void
) async throws {
    let topic = "market-data.tick.\(symbol)"

    try await nats.subscribe(topic) { message in
        do {
            let decoder = JSONDecoder()
            decoder.dateDecodingStrategy = .iso8601
            let natsMessage = try decoder.decode(
                NATSMessage<MarketDataTickPayload>.self,
                from: message.payload
            )

            if natsMessage.type == "MarketDataTick" {
                callback(natsMessage.payload)
            }
        } catch {
            print("Error parsing message: \(error)")
        }
    }

    print("Subscribed to \(topic)")
}
```

### Wildcard Subscriptions

```swift
func subscribeAllSignals(nats: NATS) async throws {
    let topic = "strategy.signal.>"

    try await nats.subscribe(topic) { message in
        do {
            let decoder = JSONDecoder()
            let natsMessage = try decoder.decode(
                NATSMessage<StrategySignalPayload>.self,
                from: message.payload
            )

            print("Signal received: \(natsMessage.payload.symbol) @ \(natsMessage.payload.price)")
        } catch {
            print("Error parsing signal: \(error)")
        }
    }
}
```

## Topic Constants

### Module: `NATSTopics.swift`

```swift
enum NATSTopics {
    enum MarketData {
        static func tick(symbol: String) -> String {
            return "market-data.tick.\(symbol)"
        }

        static func candle(symbol: String) -> String {
            return "market-data.candle.\(symbol)"
        }

        static let all = "market-data.>"
    }

    enum Strategy {
        static func signal(symbol: String) -> String {
            return "strategy.signal.\(symbol)"
        }

        static func decision(symbol: String) -> String {
            return "strategy.decision.\(symbol)"
        }

        static let allSignals = "strategy.signal.>"
        static let allDecisions = "strategy.decision.>"
    }

    enum DLQ {
        static func deadLetter(component: String, errorType: String) -> String {
            return "system.dlq.\(component).\(errorType)"
        }
    }
}
```

## SwiftUI Integration

### Observable NATS Manager

```swift
import SwiftUI
import Combine

@MainActor
class NATSObservable: ObservableObject {
    @Published var connected = false
    @Published var marketData: [String: MarketDataTickPayload] = [:]

    private var nats: NATS?
    private var cancellables = Set<AnyCancellable>()

    func connect(url: String = "nats://localhost:4222") async {
        do {
            let options = NATSOptions()
            options.url = url
            nats = try await NATS.connect(options: options)
            connected = true

            // Subscribe to market data
            try await subscribeMarketData()
        } catch {
            print("Failed to connect: \(error)")
            connected = false
        }
    }

    private func subscribeMarketData() async throws {
        guard let nats = nats else { return }

        let topic = NATSTopics.MarketData.all

        try await nats.subscribe(topic) { [weak self] message in
            Task { @MainActor in
                do {
                    let decoder = JSONDecoder()
                    let natsMessage = try decoder.decode(
                        NATSMessage<MarketDataTickPayload>.self,
                        from: message.payload
                    )

                    if natsMessage.type == "MarketDataTick" {
                        self?.marketData[natsMessage.payload.symbol] = natsMessage.payload
                    }
                } catch {
                    print("Error: \(error)")
                }
            }
        }
    }

    func disconnect() {
        nats?.close()
        nats = nil
        connected = false
    }
}
```

### SwiftUI View

```swift
import SwiftUI

struct MarketDataView: View {
    @StateObject private var nats = NATSObservable()

    var body: some View {
        VStack {
            if nats.connected {
                Text("Connected to NATS")
                    .foregroundColor(.green)
            } else {
                Text("Disconnected")
                    .foregroundColor(.red)
            }

            List(nats.marketData.keys.sorted(), id: \.self) { symbol in
                if let tick = nats.marketData[symbol] {
                    HStack {
                        Text(symbol)
                        Spacer()
                        Text(String(format: "%.2f", tick.price))
                    }
                }
            }
        }
        .onAppear {
            Task {
                await nats.connect()
            }
        }
        .onDisappear {
            nats.disconnect()
        }
    }
}
```

## Error Handling

### DLQ Integration

```swift
struct DeadLetterMessage: Codable {
    let id: String
    let timestamp: String
    let originalTopic: String
    let component: String
    let errorType: String
    let errorMessage: String
    let retryCount: Int
    let originalPayload: [String: AnyCodable]
}

func sendToDLQ(
    nats: NATS,
    topic: String,
    message: any Codable,
    errorType: String,
    retryCount: Int = 0
) async throws {
    let dlqTopic = NATSTopics.DLQ.deadLetter("swift-ipad", errorType)

    let dlqMessage = DeadLetterMessage(
        id: UUID().uuidString,
        timestamp: ISO8601DateFormatter().string(from: Date()),
        originalTopic: topic,
        component: "swift-ipad",
        errorType: errorType,
        errorMessage: "Failed to publish to \(topic)",
        retryCount: retryCount,
        originalPayload: [:]  // Extract from message
    )

    let encoder = JSONEncoder()
    let data = try encoder.encode(dlqMessage)

    try await nats.publish(dlqTopic, payload: data)
}
```

## Configuration

### Environment Variables

```swift
struct NATSConfig {
    static let url = ProcessInfo.processInfo.environment["NATS_URL"] ?? "nats://localhost:4222"
    static let enabled = ProcessInfo.processInfo.environment["NATS_ENABLED"] != "false"
    static let dlqEnabled = ProcessInfo.processInfo.environment["NATS_DLQ_ENABLED"] != "false"
}
```

## Best Practices

1. **Async/Await**: Use async/await for all NATS operations
2. **Error Handling**: Always use try/catch for NATS operations
3. **Type Safety**: Use Codable for message types
4. **Topic Constants**: Use topic helper functions
5. **Connection Management**: Handle connection lifecycle properly
6. **UI Updates**: Use @MainActor for UI updates from NATS callbacks

## References

- [Swift NATS Client](https://github.com/nats-io/swift-nats)
- [NATS Topics Registry](../NATS_TOPICS_REGISTRY.md)
- [Message Schemas](../message_schemas/README.md)
