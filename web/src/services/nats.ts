/**
 * NATS Service - WebSocket connection to NATS message queue
 *
 * Provides real-time market data and strategy updates for the frontend.
 */

import { connect, NatsConnection, Subscription } from 'nats.ws';

export interface MarketDataTick {
  symbol: string;
  bid: number;
  ask: number;
  timestamp: string;
}

export interface StrategySignal {
  symbol: string;
  price: number;
  signal_type: string;
  timestamp: string;
}

export interface StrategyDecision {
  symbol: string;
  quantity: number;
  side: string;
  mark: number;
  decision_type: string;
  timestamp: string;
}

interface NATSMessage<T> {
  id: string;
  timestamp: string;
  source: string;
  type: string;
  payload: T;
}

export class NATSService {
  private nc: NatsConnection | null = null;
  private subscriptions: Map<string, Subscription> = new Map();
  private connected: boolean = false;
  private url: string;

  constructor(url: string = 'ws://localhost:8080') {
    this.url = url;
  }

  /**
   * Connect to NATS server via WebSocket
   */
  async connect(): Promise<boolean> {
    try {
      this.nc = await connect({
        servers: [this.url],
        reconnect: true,
        reconnectTimeWait: 2000, // 2 seconds
        maxReconnectAttempts: -1, // Unlimited
      });

      this.connected = true;
      console.log(`Connected to NATS at ${this.url}`);
      return true;
    } catch (error) {
      console.error('Failed to connect to NATS:', error);
      this.connected = false;
      return false;
    }
  }

  /**
   * Disconnect from NATS server
   */
  async disconnect(): Promise<void> {
    if (this.nc) {
      try {
        // Unsubscribe from all topics
        for (const sub of this.subscriptions.values()) {
          await sub.unsubscribe();
        }
        this.subscriptions.clear();

        await this.nc.close();
        this.connected = false;
        console.log('Disconnected from NATS');
      } catch (error) {
        console.error('Error disconnecting from NATS:', error);
      }
    }
  }

  /**
   * Check if connected to NATS
   */
  isConnected(): boolean {
    return this.connected && this.nc !== null;
  }

  /**
   * Subscribe to market data ticks
   *
   * @param callback Function to call when market data received
   * @param symbol Optional specific symbol, or undefined for all symbols
   */
  async subscribeMarketData(
    callback: (data: MarketDataTick) => void,
    symbol?: string
  ): Promise<string | null> {
    if (!this.isConnected() || !this.nc) {
      console.warn('Not connected to NATS - cannot subscribe');
      return null;
    }

    const topic = symbol ? `market-data.tick.${symbol}` : 'market-data.tick.>';

    try {
      const sub = this.nc.subscribe(topic, {
        callback: (err, msg) => {
          if (err) {
            console.error(`Error receiving message on ${topic}:`, err);
            return;
          }

          try {
            const data = JSON.parse(new TextDecoder().decode(msg.data)) as NATSMessage<MarketDataTick>;
            // Extract payload from NATS message format
            const payload = data.payload || (data as unknown as MarketDataTick);
            callback(payload);
          } catch (e) {
            console.error(`Error parsing market data message:`, e);
          }
        },
      });

      this.subscriptions.set(topic, sub);
      console.log(`Subscribed to ${topic}`);
      return topic;
    } catch (error) {
      console.error(`Failed to subscribe to ${topic}:`, error);
      return null;
    }
  }

  /**
   * Subscribe to strategy signals
   *
   * @param callback Function to call when strategy signal received
   * @param symbol Optional specific symbol, or undefined for all symbols
   */
  async subscribeStrategySignals(
    callback: (signal: StrategySignal) => void,
    symbol?: string
  ): Promise<string | null> {
    if (!this.isConnected() || !this.nc) {
      console.warn('Not connected to NATS - cannot subscribe');
      return null;
    }

    const topic = symbol ? `strategy.signal.${symbol}` : 'strategy.signal.>';

    try {
      const sub = this.nc.subscribe(topic, {
        callback: (err, msg) => {
          if (err) {
            console.error(`Error receiving message on ${topic}:`, err);
            return;
          }

          try {
            const data = JSON.parse(new TextDecoder().decode(msg.data)) as NATSMessage<StrategySignal>;
            const payload = data.payload || (data as unknown as StrategySignal);
            callback(payload);
          } catch (e) {
            console.error(`Error parsing strategy signal message:`, e);
          }
        },
      });

      this.subscriptions.set(topic, sub);
      console.log(`Subscribed to ${topic}`);
      return topic;
    } catch (error) {
      console.error(`Failed to subscribe to ${topic}:`, error);
      return null;
    }
  }

  /**
   * Subscribe to strategy decisions
   *
   * @param callback Function to call when strategy decision received
   * @param symbol Optional specific symbol, or undefined for all symbols
   */
  async subscribeStrategyDecisions(
    callback: (decision: StrategyDecision) => void,
    symbol?: string
  ): Promise<string | null> {
    if (!this.isConnected() || !this.nc) {
      console.warn('Not connected to NATS - cannot subscribe');
      return null;
    }

    const topic = symbol ? `strategy.decision.${symbol}` : 'strategy.decision.>';

    try {
      const sub = this.nc.subscribe(topic, {
        callback: (err, msg) => {
          if (err) {
            console.error(`Error receiving message on ${topic}:`, err);
            return;
          }

          try {
            const data = JSON.parse(new TextDecoder().decode(msg.data)) as NATSMessage<StrategyDecision>;
            const payload = data.payload || (data as unknown as StrategyDecision);
            callback(payload);
          } catch (e) {
            console.error(`Error parsing strategy decision message:`, e);
          }
        },
      });

      this.subscriptions.set(topic, sub);
      console.log(`Subscribed to ${topic}`);
      return topic;
    } catch (error) {
      console.error(`Failed to subscribe to ${topic}:`, error);
      return null;
    }
  }

  /**
   * Unsubscribe from a topic
   */
  async unsubscribe(topic: string): Promise<void> {
    const sub = this.subscriptions.get(topic);
    if (sub) {
      try {
        await sub.unsubscribe();
        this.subscriptions.delete(topic);
        console.log(`Unsubscribed from ${topic}`);
      } catch (error) {
        console.error(`Error unsubscribing from ${topic}:`, error);
      }
    }
  }

  /**
   * Unsubscribe from all topics
   */
  async unsubscribeAll(): Promise<void> {
    for (const topic of this.subscriptions.keys()) {
      await this.unsubscribe(topic);
    }
  }
}

// Singleton instance
let natsServiceInstance: NATSService | null = null;

/**
 * Get or create NATS service singleton
 */
export function getNATSService(url?: string): NATSService {
  if (!natsServiceInstance) {
    natsServiceInstance = new NATSService(url);
  }
  return natsServiceInstance;
}
