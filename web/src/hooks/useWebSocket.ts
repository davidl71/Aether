import { useEffect, useState, useRef, useCallback } from 'react';
import { snapshotClient, type ConnectionStatus } from '../api/snapshot';

/**
 * WebSocket message types
 */
export type WebSocketMessage =
  | { type: 'snapshot'; data: unknown }
  | { type: 'order_filled'; data: unknown }
  | { type: 'position_updated'; data: unknown }
  | { type: 'symbol_updated'; data: unknown }
  | { type: 'connected'; data?: unknown };

/**
 * Hook options for useWebSocket
 */
export interface UseWebSocketOptions {
  reconnect?: boolean;
  reconnectInterval?: number;
  maxReconnectAttempts?: number;
  onMessage?: (message: WebSocketMessage) => void;
  onError?: (error: Error) => void;
  onConnect?: () => void;
  onDisconnect?: () => void;
  url?: string;
}

/**
 * Hook to manage WebSocket connection with automatic reconnection
 */
export function useWebSocket(options: UseWebSocketOptions = {}) {
  const {
    reconnect = true,
    reconnectInterval = 3000,
    maxReconnectAttempts = 10,
    onMessage,
    onError,
    onConnect,
    onDisconnect,
    url,
  } = options;

  const [connected, setConnected] = useState(false);
  const [connecting, setConnecting] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectAttemptsRef = useRef(0);
  const reconnectTimeoutRef = useRef<number | null>(null);

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      return; // Already connected
    }

    if (wsRef.current?.readyState === WebSocket.CONNECTING) {
      return; // Already connecting
    }

    try {
      setConnecting(true);
      setError(null);

      const wsUrl = url || (typeof window !== 'undefined' ? `ws://${window.location.hostname}:8000/ws` : '');
      const ws = new WebSocket(wsUrl);
      wsRef.current = ws;

      ws.onopen = () => {
        setConnected(true);
        setConnecting(false);
        reconnectAttemptsRef.current = 0;
        onConnect?.();
      };

      ws.onmessage = (event) => {
        try {
          const message = JSON.parse(event.data) as WebSocketMessage;
          onMessage?.(message);
        } catch (err) {
          const error = err instanceof Error ? err : new Error('Failed to parse WebSocket message');
          setError(error);
          onError?.(error);
        }
      };

      ws.onerror = (event) => {
        const error = new Error('WebSocket error');
        setError(error);
        setConnecting(false);
        onError?.(error);
      };

      ws.onclose = () => {
        setConnected(false);
        setConnecting(false);
        onDisconnect?.();

        // Attempt reconnection if enabled
        if (reconnect && reconnectAttemptsRef.current < maxReconnectAttempts) {
          reconnectAttemptsRef.current++;
          reconnectTimeoutRef.current = window.setTimeout(() => {
            connect();
          }, reconnectInterval);
        } else if (reconnectAttemptsRef.current >= maxReconnectAttempts) {
          setError(new Error('Max reconnection attempts reached'));
        }
      };
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Failed to create WebSocket');
      setError(error);
      setConnecting(false);
      onError?.(error);
    }
  }, [url, reconnect, reconnectInterval, maxReconnectAttempts, onMessage, onError, onConnect, onDisconnect]);

  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current !== null) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }
    setConnected(false);
    setConnecting(false);
  }, []);

  useEffect(() => {
    connect();
    return () => {
      disconnect();
    };
  }, [connect, disconnect]);

  return {
    connected,
    connecting,
    error,
    connect,
    disconnect,
  };
}

/**
 * Hook to monitor WebSocket connection status
 * Exposes connection state for UI indicators
 */
export function useWebSocketStatus() {
  const [status, setStatus] = useState<ConnectionStatus>(snapshotClient.getConnectionStatus());

  useEffect(() => {
    const unsubscribe = snapshotClient.onStatusChange((newStatus) => {
      setStatus(newStatus);
    });

    return unsubscribe;
  }, []);

  return { status };
}
