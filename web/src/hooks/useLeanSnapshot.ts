/**
 * useLeanSnapshot.ts - React hook for LEAN snapshot with WebSocket support
 *
 * Combines REST API polling with WebSocket real-time updates for optimal performance.
 * Falls back to polling if WebSocket is unavailable.
 */

import { useEffect, useState, useCallback } from 'react';
import { leanClient } from '../api/leanClient';
import { useWebSocket, type WebSocketMessage } from './useWebSocket';
import type { SnapshotPayload } from '../types/snapshot';

export interface UseLeanSnapshotOptions {
  useWebSocket?: boolean;
  pollInterval?: number;
  onError?: (error: Error) => void;
}

export interface LeanSnapshotState {
  snapshot: SnapshotPayload | null;
  isLoading: boolean;
  error: string | null;
  source: 'websocket' | 'polling' | 'none';
}

export function useLeanSnapshot(options: UseLeanSnapshotOptions = {}) {
  const {
    useWebSocket: enableWebSocket = true,
    pollInterval = 2000,
    onError,
  } = options;

  const [state, setState] = useState<LeanSnapshotState>({
    snapshot: null,
    isLoading: true,
    error: null,
    source: 'none',
  });

  const [pollingEnabled, setPollingEnabled] = useState(!enableWebSocket);
  const pollingTimerRef = useState<number | null>(null)[0];

  // Fetch snapshot via REST API
  const fetchSnapshot = useCallback(async () => {
    try {
      const snapshot = await leanClient.getSnapshot();
      setState((prev) => ({
        snapshot,
        isLoading: false,
        error: null,
        source: prev.source === 'websocket' ? 'websocket' : 'polling',
      }));
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to fetch snapshot';
      setState((prev) => ({
        ...prev,
        isLoading: false,
        error: errorMessage,
      }));
      onError?.(error instanceof Error ? error : new Error(errorMessage));
    }
  }, [onError]);

  // WebSocket hook for real-time updates
  const ws = useWebSocket({
    reconnect: true,
    onMessage: (message: WebSocketMessage) => {
      if (message.type === 'snapshot') {
        // Full snapshot update
        setState((prev) => ({
          snapshot: message.data as SnapshotPayload,
          isLoading: false,
          error: null,
          source: 'websocket',
        }));
        // Disable polling when WebSocket is working
        setPollingEnabled(false);
      } else if (message.type === 'order_filled' || message.type === 'position_updated') {
        // Partial update - refetch full snapshot
        void fetchSnapshot();
      }
    },
    onError: (error) => {
      // Fall back to polling if WebSocket fails
      setPollingEnabled(true);
      setState((prev) => ({
        ...prev,
        error: `WebSocket error: ${error.message}. Falling back to polling.`,
        source: 'polling',
      }));
    },
    onConnect: () => {
      // Fetch initial snapshot when WebSocket connects
      void fetchSnapshot();
      setState((prev) => ({ ...prev, source: 'websocket' }));
    },
  });

  // Polling fallback
  useEffect(() => {
    if (!pollingEnabled) {
      return;
    }

    // Initial fetch
    void fetchSnapshot();

    // Set up polling interval
    const interval = window.setInterval(() => {
      void fetchSnapshot();
    }, pollInterval);

    return () => {
      clearInterval(interval);
    };
  }, [pollingEnabled, pollInterval, fetchSnapshot]);

  // If WebSocket is enabled but not connected, enable polling as fallback
  useEffect(() => {
    if (enableWebSocket && !ws.connected && !ws.connecting && !pollingEnabled) {
      setPollingEnabled(true);
    }
  }, [enableWebSocket, ws.connected, ws.connecting, pollingEnabled]);

  return {
    ...state,
    websocket: {
      connected: ws.connected,
      connecting: ws.connecting,
      error: ws.error,
    },
    refetch: fetchSnapshot,
  };
}
