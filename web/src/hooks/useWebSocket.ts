/**
 * useWebSocket.ts - React hook for WebSocket connections
 *
 * Provides real-time updates from LEAN via WebSocket.
 */

import { useEffect, useState, useRef, useCallback } from 'react';
import { leanClient } from '../api/leanClient';
import type { SnapshotPayload } from '../types/snapshot';

export interface WebSocketMessage {
  type: string;
  data: unknown;
  timestamp: string;
}

export interface WebSocketState {
  connected: boolean;
  connecting: boolean;
  error: string | null;
  lastMessage: WebSocketMessage | null;
  snapshot: SnapshotPayload | null;
}

export interface UseWebSocketOptions {
  url?: string;
  reconnect?: boolean;
  reconnectInterval?: number;
  maxReconnectAttempts?: number;
  onMessage?: (message: WebSocketMessage) => void;
  onError?: (error: Error) => void;
  onConnect?: () => void;
  onDisconnect?: () => void;
}

const DEFAULT_RECONNECT_INTERVAL = 3000; // 3 seconds
const DEFAULT_MAX_RECONNECT_ATTEMPTS = 10;

export function useWebSocket(options: UseWebSocketOptions = {}) {
  const {
    url,
    reconnect = true,
    reconnectInterval = DEFAULT_RECONNECT_INTERVAL,
    maxReconnectAttempts = DEFAULT_MAX_RECONNECT_ATTEMPTS,
    onMessage,
    onError,
    onConnect,
    onDisconnect,
  } = options;

  const [state, setState] = useState<WebSocketState>({
    connected: false,
    connecting: false,
    error: null,
    lastMessage: null,
    snapshot: null,
  });

  const wsRef = useRef<WebSocket | null>(null);
  const reconnectAttemptsRef = useRef(0);
  const reconnectTimeoutRef = useRef<number | null>(null);
  const shouldReconnectRef = useRef(reconnect);

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      return; // Already connected
    }

    const wsUrl = url ?? leanClient.getWebSocketUrl();
    setState((prev) => ({ ...prev, connecting: true, error: null }));

    try {
      const ws = new WebSocket(wsUrl);
      wsRef.current = ws;

      ws.onopen = () => {
        setState((prev) => ({
          ...prev,
          connected: true,
          connecting: false,
          error: null,
        }));
        reconnectAttemptsRef.current = 0;
        onConnect?.();
      };

      ws.onmessage = (event) => {
        try {
          const message = JSON.parse(event.data) as WebSocketMessage;

          setState((prev) => ({
            ...prev,
            lastMessage: message,
            // Update snapshot if message type is snapshot
            snapshot: message.type === 'snapshot' ? (message.data as SnapshotPayload) : prev.snapshot,
          }));

          onMessage?.(message);
        } catch (error) {
          console.error('Failed to parse WebSocket message:', error);
          setState((prev) => ({
            ...prev,
            error: error instanceof Error ? error.message : 'Failed to parse message',
          }));
        }
      };

      ws.onerror = (error) => {
        const errorMessage = 'WebSocket connection error';
        setState((prev) => ({
          ...prev,
          error: errorMessage,
          connecting: false,
        }));
        onError?.(new Error(errorMessage));
      };

      ws.onclose = () => {
        setState((prev) => ({
          ...prev,
          connected: false,
          connecting: false,
        }));
        onDisconnect?.();

        // Attempt reconnection if enabled
        if (shouldReconnectRef.current && reconnectAttemptsRef.current < maxReconnectAttempts) {
          reconnectAttemptsRef.current += 1;
          reconnectTimeoutRef.current = window.setTimeout(() => {
            connect();
          }, reconnectInterval);
        } else if (reconnectAttemptsRef.current >= maxReconnectAttempts) {
          setState((prev) => ({
            ...prev,
            error: `Failed to reconnect after ${maxReconnectAttempts} attempts`,
          }));
        }
      };
    } catch (error) {
      setState((prev) => ({
        ...prev,
        connecting: false,
        error: error instanceof Error ? error.message : 'Failed to create WebSocket',
      }));
      onError?.(error instanceof Error ? error : new Error('Failed to create WebSocket'));
    }
  }, [url, reconnectInterval, maxReconnectAttempts, onMessage, onError, onConnect, onDisconnect]);

  const disconnect = useCallback(() => {
    shouldReconnectRef.current = false;
    if (reconnectTimeoutRef.current !== null) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }
  }, []);

  const send = useCallback((data: string | object) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      const message = typeof data === 'string' ? data : JSON.stringify(data);
      wsRef.current.send(message);
    } else {
      throw new Error('WebSocket is not connected');
    }
  }, []);

  useEffect(() => {
    connect();
    return () => {
      disconnect();
    };
  }, [connect, disconnect]);

  return {
    ...state,
    connect,
    disconnect,
    send,
  };
}
