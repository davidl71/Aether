/**
 * React hook for NATS message queue integration
 *
 * Provides real-time market data and strategy updates via NATS subscriptions.
 */

import { useEffect, useState, useCallback, useRef } from 'react';
import { getNATSService, NATSService, MarketDataTick, StrategySignal, StrategyDecision } from '../services/nats';

interface UseNATSOptions {
  natsUrl?: string;
  autoConnect?: boolean;
  subscribeMarketData?: boolean;
  subscribeStrategySignals?: boolean;
  subscribeStrategyDecisions?: boolean;
  symbol?: string; // Optional specific symbol filter
}

interface UseNATSReturn {
  connected: boolean;
  marketData: MarketDataTick | null;
  strategySignals: StrategySignal[];
  strategyDecisions: StrategyDecision[];
  connect: () => Promise<boolean>;
  disconnect: () => Promise<void>;
  error: Error | null;
}

export function useNATS(options: UseNATSOptions = {}): UseNATSReturn {
  const {
    natsUrl,
    autoConnect = true,
    subscribeMarketData = true,
    subscribeStrategySignals = true,
    subscribeStrategyDecisions = true,
    symbol,
  } = options;

  const [connected, setConnected] = useState(false);
  const [marketData, setMarketData] = useState<MarketDataTick | null>(null);
  const [strategySignals, setStrategySignals] = useState<StrategySignal[]>([]);
  const [strategyDecisions, setStrategyDecisions] = useState<StrategyDecision[]>([]);
  const [error, setError] = useState<Error | null>(null);

  const natsServiceRef = useRef<NATSService | null>(null);
  const subscriptionsRef = useRef<string[]>([]);

  const connect = useCallback(async (): Promise<boolean> => {
    try {
      const service = getNATSService(natsUrl);
      natsServiceRef.current = service;

      const success = await service.connect();
      setConnected(success);
      setError(null);

      if (success) {
        // Subscribe to topics
        if (subscribeMarketData) {
          const topic = await service.subscribeMarketData(
            (data) => {
              setMarketData(data);
            },
            symbol
          );
          if (topic) subscriptionsRef.current.push(topic);
        }

        if (subscribeStrategySignals) {
          const topic = await service.subscribeStrategySignals(
            (signal) => {
              setStrategySignals((prev) => [...prev.slice(-99), signal]); // Keep last 100
            },
            symbol
          );
          if (topic) subscriptionsRef.current.push(topic);
        }

        if (subscribeStrategyDecisions) {
          const topic = await service.subscribeStrategyDecisions(
            (decision) => {
              setStrategyDecisions((prev) => [...prev.slice(-99), decision]); // Keep last 100
            },
            symbol
          );
          if (topic) subscriptionsRef.current.push(topic);
        }
      }

      return success;
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      setConnected(false);
      return false;
    }
  }, [natsUrl, subscribeMarketData, subscribeStrategySignals, subscribeStrategyDecisions, symbol]);

  const disconnect = useCallback(async (): Promise<void> => {
    if (natsServiceRef.current) {
      await natsServiceRef.current.disconnect();
      subscriptionsRef.current = [];
      setConnected(false);
      setMarketData(null);
      setStrategySignals([]);
      setStrategyDecisions([]);
    }
  }, []);

  // Auto-connect on mount if enabled
  useEffect(() => {
    if (autoConnect) {
      connect();
    }

    // Cleanup on unmount
    return () => {
      disconnect();
    };
  }, [autoConnect, connect, disconnect]);

  return {
    connected,
    marketData,
    strategySignals,
    strategyDecisions,
    connect,
    disconnect,
    error,
  };
}
