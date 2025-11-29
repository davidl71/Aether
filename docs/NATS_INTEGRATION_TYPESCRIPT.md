# NATS Integration Guide - TypeScript

**Date**: 2025-11-20
**Status**: Planning/Implementation Guide
**Library**: `nats.ws` (WebSocket NATS client for browsers) or `nats` (Node.js)

## Overview

This guide covers integrating NATS message queue into TypeScript frontend applications for real-time updates via WebSocket or Node.js connections.

## Prerequisites

### Installation

```bash
# For Node.js backend
npm install nats

# For browser/WebSocket
npm install nats.ws
```

## Connection Management

### Node.js Connection

```typescript
import { connect, NatsConnection } from 'nats';

async function connectNATS(url: string = 'nats://localhost:4222'): Promise<NatsConnection | null> {
  try {
    const nc = await connect({
      servers: [url],
      reconnect: true,
      reconnectTimeWait: 2000,  // 2 seconds
      maxReconnectAttempts: -1,  // Unlimited
    });

    console.log(`Connected to NATS at ${url}`);
    return nc;
  } catch (error) {
    console.error('Failed to connect to NATS:', error);
    return null;
  }
}
```

### Browser/WebSocket Connection

```typescript
import { connect } from 'nats.ws';

async function connectNATSWs(url: string = 'ws://localhost:8080'): Promise<NatsConnection | null> {
  try {
    const nc = await connect({
      servers: [url],
      reconnect: true,
    });

    console.log(`Connected to NATS via WebSocket at ${url}`);
    return nc;
  } catch (error) {
    console.error('Failed to connect to NATS:', error);
    return null;
  }
}
```

## Publishing Messages

### Publish Strategy Signal

```typescript
import { NatsConnection } from 'nats';
import { v4 as uuidv4 } from 'uuid';

interface StrategySignalPayload {
  symbol: string;
  price: number;
  timestamp: string;
}

async function publishStrategySignal(
  nc: NatsConnection,
  symbol: string,
  price: number
): Promise<void> {
  const topic = `strategy.signal.${symbol}`;

  const message = {
    id: uuidv4(),
    timestamp: new Date().toISOString(),
    source: 'typescript-frontend',
    type: 'StrategySignal',
    payload: {
      symbol,
      price,
      timestamp: new Date().toISOString(),
    } as StrategySignalPayload,
  };

  try {
    nc.publish(topic, JSON.stringify(message));
    console.log(`Published signal for ${symbol} at ${price}`);
  } catch (error) {
    console.error('Failed to publish signal:', error);
    await sendToDLQ(nc, topic, message, 'publish_error');
  }
}
```

## Subscribing to Messages

### Subscribe to Market Data

```typescript
import { NatsConnection, Subscription } from 'nats';

interface MarketDataTick {
  symbol: string;
  bid?: number;
  ask?: number;
  price: number;
}

async function subscribeMarketData(
  nc: NatsConnection,
  symbol: string,
  callback: (data: MarketDataTick) => void
): Promise<Subscription> {
  const topic = `market-data.tick.${symbol}`;

  const sub = nc.subscribe(topic, {
    callback: (err, msg) => {
      if (err) {
        console.error('Error receiving message:', err);
        return;
      }

      try {
        const data = JSON.parse(msg.string());

        if (data.type === 'MarketDataTick') {
          callback(data.payload as MarketDataTick);
        }
      } catch (error) {
        console.error('Error parsing message:', error);
      }
    },
  });

  console.log(`Subscribed to ${topic}`);
  return sub;
}

// Usage
const nc = await connectNATS();
const sub = await subscribeMarketData(nc, 'SPY', (data) => {
  console.log(`${data.symbol}: bid=${data.bid}, ask=${data.ask}, price=${data.price}`);
  updateUI(data);
});
```

### Wildcard Subscriptions

```typescript
async function subscribeAllSignals(nc: NatsConnection): Promise<Subscription> {
  const topic = 'strategy.signal.>';

  const sub = nc.subscribe(topic, {
    callback: (err, msg) => {
      if (err) {
        console.error('Error:', err);
        return;
      }

      const data = JSON.parse(msg.string());
      console.log('Signal received:', data.payload);
    },
  });

  return sub;
}
```

## Topic Constants

### Module: `nats-topics.ts`

```typescript
export namespace NATSTopics {
  export namespace MarketData {
    export function tick(symbol: string): string {
      return `market-data.tick.${symbol}`;
    }

    export function candle(symbol: string): string {
      return `market-data.candle.${symbol}`;
    }

    export const ALL = 'market-data.>';
  }

  export namespace Strategy {
    export function signal(symbol: string): string {
      return `strategy.signal.${symbol}`;
    }

    export function decision(symbol: string): string {
      return `strategy.decision.${symbol}`;
    }

    export const ALL_SIGNALS = 'strategy.signal.>';
    export const ALL_DECISIONS = 'strategy.decision.>';
  }

  export namespace DLQ {
    export function deadLetter(component: string, errorType: string): string {
      return `system.dlq.${component}.${errorType}`;
    }
  }
}
```

## Type Definitions

### Message Types

```typescript
// types/nats-messages.ts

export interface NATSMessage<T> {
  id: string;
  timestamp: string;
  source: string;
  type: string;
  payload: T;
}

export interface MarketDataTickPayload {
  symbol: string;
  price: number;
  size?: number;
  tick_type?: string;
  bid?: number;
  ask?: number;
  spread?: number;
}

export interface StrategySignalPayload {
  symbol: string;
  price: number;
  timestamp: string;
}

export interface StrategyDecisionPayload {
  symbol: string;
  quantity: number;
  side: 'BUY' | 'SELL';
  mark: number;
  strategy_name?: string;
  confidence?: number;
}

export type MarketDataTickMessage = NATSMessage<MarketDataTickPayload>;
export type StrategySignalMessage = NATSMessage<StrategySignalPayload>;
export type StrategyDecisionMessage = NATSMessage<StrategyDecisionPayload>;
```

## React Integration

### Hook for NATS Connection

```typescript
import { useEffect, useState } from 'react';
import { connect, NatsConnection } from 'nats.ws';

export function useNATSConnection(url: string = 'ws://localhost:8080') {
  const [nc, setNc] = useState<NatsConnection | null>(null);
  const [connected, setConnected] = useState(false);

  useEffect(() => {
    let connection: NatsConnection | null = null;

    connect({ servers: [url] })
      .then((conn) => {
        connection = conn;
        setNc(conn);
        setConnected(true);
      })
      .catch((error) => {
        console.error('NATS connection error:', error);
        setConnected(false);
      });

    return () => {
      if (connection) {
        connection.close();
      }
    };
  }, [url]);

  return { nc, connected };
}
```

### Hook for Market Data

```typescript
import { useEffect, useState } from 'react';
import { useNATSConnection } from './useNATSConnection';
import { MarketDataTickPayload } from './types/nats-messages';

export function useMarketData(symbol: string) {
  const { nc, connected } = useNATSConnection();
  const [tick, setTick] = useState<MarketDataTickPayload | null>(null);

  useEffect(() => {
    if (!nc || !connected) return;

    const topic = `market-data.tick.${symbol}`;
    const sub = nc.subscribe(topic, {
      callback: (err, msg) => {
        if (err) return;

        const data = JSON.parse(msg.string());
        if (data.type === 'MarketDataTick') {
          setTick(data.payload);
        }
      },
    });

    return () => {
      sub.unsubscribe();
    };
  }, [nc, connected, symbol]);

  return tick;
}
```

## Error Handling

### DLQ Integration

```typescript
async function sendToDLQ(
  nc: NatsConnection,
  originalTopic: string,
  message: any,
  errorType: string,
  retryCount: number = 0
): Promise<void> {
  const dlqTopic = NATSTopics.DLQ.deadLetter('typescript-frontend', errorType);

  const dlqMessage = {
    id: uuidv4(),
    timestamp: new Date().toISOString(),
    original_topic: originalTopic,
    component: 'typescript-frontend',
    error_type: errorType,
    error_message: `Failed to publish to ${originalTopic}`,
    retry_count: retryCount,
    original_payload: message.payload,
  };

  try {
    nc.publish(dlqTopic, JSON.stringify(dlqMessage));
  } catch (error) {
    console.error('Failed to send to DLQ:', error);
  }
}
```

## Configuration

### Environment Variables

```typescript
// config/nats.ts
export const NATS_CONFIG = {
  url: process.env.NEXT_PUBLIC_NATS_URL || 'ws://localhost:8080',
  enabled: process.env.NEXT_PUBLIC_NATS_ENABLED !== 'false',
  dlqEnabled: process.env.NEXT_PUBLIC_NATS_DLQ_ENABLED !== 'false',
};
```

## Best Practices

1. **Connection Management**: Reuse connection, handle reconnection
2. **Type Safety**: Use TypeScript types for all messages
3. **Error Handling**: Always handle errors in callbacks
4. **Topic Constants**: Use topic helper functions
5. **Unsubscribe**: Always unsubscribe when component unmounts
6. **Message Validation**: Validate message structure before processing

## References

- [NATS TypeScript Client](https://github.com/nats-io/nats.deno)
- [NATS Topics Registry](research/../NATS_TOPICS_REGISTRY.md)
- [Message Schemas](research/../message_schemas/README.md)
