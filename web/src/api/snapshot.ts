import type { SnapshotPayload } from '../types/snapshot';

const DEFAULT_ENDPOINT = '/data/snapshot.json';
const POLL_INTERVAL = 2000;
const DEFAULT_WS_RECONNECT_INTERVAL = 3000;
const DEFAULT_MAX_RECONNECT_ATTEMPTS = 10;

export interface SnapshotClientOptions {
  endpoint?: string;
  pollIntervalMs?: number;
  useLeanApi?: boolean; // Use LEAN API wrapper instead of Rust backend
  useWebSocket?: boolean; // Use WebSocket when available (default: true)
  wsUrl?: string; // WebSocket URL (auto-detected if not provided)
}

export type SnapshotListener = (payload: SnapshotPayload) => void;
export type SnapshotErrorListener = (error: Error) => void;

export type ConnectionStatus = 'connecting' | 'connected' | 'disconnected' | 'error' | 'polling';

export class SnapshotClient {
  private readonly endpoint: string;
  private readonly pollInterval: number;
  private readonly useLeanApi: boolean;
  private readonly useWebSocket: boolean;
  private readonly wsUrl: string | null;
  private timer: number | null = null;
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private reconnectTimeout: number | null = null;
  private listeners: Map<SnapshotListener, SnapshotErrorListener | undefined> =
    new Map<SnapshotListener, SnapshotErrorListener | undefined>();
  private isPolling = false;
  private isWebSocketConnected = false;
  private connectionStatus: ConnectionStatus = 'disconnected';
  private statusListeners: Set<(status: ConnectionStatus) => void> = new Set();

  constructor(options: SnapshotClientOptions = {}) {
    const env = (import.meta as unknown as { env?: Record<string, unknown> }).env;

    // Determine if we should use LEAN API
    this.useLeanApi = options.useLeanApi ??
      (typeof env?.VITE_USE_LEAN_API === 'string' && env.VITE_USE_LEAN_API === 'true');

    // Get endpoint based on API type
    if (this.useLeanApi) {
      // Use LEAN API endpoint
      const leanUrl = typeof env?.VITE_LEAN_API_URL === 'string' && env.VITE_LEAN_API_URL.length > 0
        ? env.VITE_LEAN_API_URL
        : 'http://localhost:8000';
      this.endpoint = options.endpoint ?? `${leanUrl}/api/v1/snapshot`;
      // LEAN API has WebSocket support at /ws
      this.wsUrl = options.wsUrl ?? (leanUrl.startsWith('https://')
        ? leanUrl.replace('https://', 'wss://')
        : leanUrl.replace('http://', 'ws://')) + '/ws';
    } else {
      // Use Rust backend endpoint
      let envEndpoint: string | undefined;
      if (typeof env?.VITE_API_URL === 'string' && env.VITE_API_URL.length > 0) {
        envEndpoint = env.VITE_API_URL;
      }
      this.endpoint = options.endpoint ?? envEndpoint ?? DEFAULT_ENDPOINT;
      // Rust backend WebSocket (if available) - construct from REST endpoint
      const baseUrl = envEndpoint ?? 'http://127.0.0.1:8080';
      this.wsUrl = options.wsUrl ?? (baseUrl.startsWith('https://')
        ? baseUrl.replace('https://', 'wss://')
        : baseUrl.replace('http://', 'ws://')) + '/ws';
    }

    this.pollInterval = options.pollIntervalMs ?? POLL_INTERVAL;
    this.useWebSocket = options.useWebSocket ?? true;
  }

  subscribe(listener: SnapshotListener, onError?: SnapshotErrorListener) {
    this.listeners.set(listener, onError);
    if (this.listeners.size === 1) {
      // Fetch initial snapshot
      void this.fetchOnce();

      // Try WebSocket first if enabled, fallback to polling
      if (this.useWebSocket && this.wsUrl) {
        this.connectWebSocket();
      } else {
        this.startPolling();
      }
    }
    return () => this.unsubscribe(listener);
  }

  unsubscribe(listener: SnapshotListener) {
    this.listeners.delete(listener);
    if (this.listeners.size === 0) {
      this.stop();
    }
  }

  /**
   * Get current connection status
   */
  getConnectionStatus(): ConnectionStatus {
    return this.connectionStatus;
  }

  /**
   * Subscribe to connection status changes
   */
  onStatusChange(listener: (status: ConnectionStatus) => void): () => void {
    this.statusListeners.add(listener);
    // Immediately notify of current status
    listener(this.connectionStatus);
    return () => {
      this.statusListeners.delete(listener);
    };
  }

  private setConnectionStatus(status: ConnectionStatus) {
    if (this.connectionStatus !== status) {
      this.connectionStatus = status;
      this.statusListeners.forEach((listener) => {
        try {
          listener(status);
        } catch (error) {
          console.error('Error in status listener:', error);
        }
      });
    }
  }

  private connectWebSocket() {
    if (!this.wsUrl || this.ws) {
      // Already connected or no WebSocket URL
      this.startPolling();
      return;
    }

    try {
      this.setConnectionStatus('connecting');
      this.ws = new WebSocket(this.wsUrl);

      this.ws.onopen = () => {
        this.isWebSocketConnected = true;
        this.reconnectAttempts = 0;
        this.setConnectionStatus('connected');
        this.stopPolling(); // Stop polling when WebSocket is connected
        // Fetch initial snapshot via REST
        void this.fetchOnce();
      };

      this.ws.onmessage = (event) => {
        try {
          const message = JSON.parse(event.data);

          // Handle different message types
          if (message.type === 'snapshot' && message.data) {
            // Full snapshot update
            const payload = message.data as SnapshotPayload;
            this.notifyListeners(payload);
          } else if (message.type === 'connected') {
            // Connection confirmation - fetch initial snapshot
            void this.fetchOnce();
          } else if (message.type === 'order_filled' || message.type === 'position_updated' || message.type === 'symbol_updated') {
            // Partial update - refetch full snapshot
            void this.fetchOnce();
          }
        } catch (error) {
          console.error('Failed to parse WebSocket message:', error);
          this.notifyError(new Error('Failed to parse WebSocket message'));
        }
      };

      this.ws.onerror = (error) => {
        // Don't log as error - WebSocket may not be available, polling is a valid fallback
        console.debug('WebSocket not available, using polling instead');
        this.isWebSocketConnected = false;
        this.setConnectionStatus('polling');
        // Fallback to polling (this is expected if WebSocket server isn't running)
        this.startPolling();
        // Don't notify as error - polling is a valid fallback
      };

      this.ws.onclose = () => {
        this.isWebSocketConnected = false;
        this.ws = null;

        // Attempt reconnection if we have listeners
        if (this.listeners.size > 0 && this.reconnectAttempts < DEFAULT_MAX_RECONNECT_ATTEMPTS) {
          this.reconnectAttempts++;
          this.setConnectionStatus('connecting');
          this.reconnectTimeout = window.setTimeout(() => {
            this.connectWebSocket();
          }, DEFAULT_WS_RECONNECT_INTERVAL);
        } else if (this.reconnectAttempts >= DEFAULT_MAX_RECONNECT_ATTEMPTS) {
          // Max reconnection attempts reached, fallback to polling
          console.debug('WebSocket reconnection limit reached. Using polling instead.');
          this.setConnectionStatus('polling');
          this.startPolling();
        } else {
          // No listeners, just fallback to polling
          this.setConnectionStatus('polling');
          this.startPolling();
        }
      };
    } catch (error) {
      // WebSocket creation failed - fallback to polling (expected if server doesn't support WebSocket)
      console.debug('WebSocket not available, using polling instead');
      this.setConnectionStatus('polling');
      this.startPolling();
    }
  }

  private startPolling() {
    if (this.isPolling) {
      return; // Already polling
    }
    this.isPolling = true;
    this.setConnectionStatus('polling');
    this.stopWebSocket(); // Stop WebSocket when polling
    this.stopPolling(); // Clear any existing polling
    this.timer = window.setInterval(() => {
      void this.fetchOnce();
    }, this.pollInterval);
  }

  private stopPolling() {
    if (this.timer !== null) {
      clearInterval(this.timer);
      this.timer = null;
    }
    this.isPolling = false;
  }

  private stopWebSocket() {
    if (this.reconnectTimeout !== null) {
      clearTimeout(this.reconnectTimeout);
      this.reconnectTimeout = null;
    }
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    this.isWebSocketConnected = false;
  }

  private start() {
    // Legacy method - use startPolling or connectWebSocket instead
    this.startPolling();
  }

  private stop() {
    this.stopPolling();
    this.stopWebSocket();
    this.reconnectAttempts = 0;
  }

  private notifyListeners(payload: SnapshotPayload) {
    this.listeners.forEach((onError, listener) => {
      try {
        listener(payload);
      } catch (error) {
        console.error('Error in snapshot listener:', error);
      }
    });
  }

  private notifyError(error: Error) {
    this.listeners.forEach((onError) => {
      if (onError) {
        try {
          onError(error);
        } catch (err) {
          console.error('Error in error listener:', err);
        }
      }
    });
  }

  private async fetchOnce() {
    try {
      const response = await fetch(this.endpoint, {
        headers: { 'cache-control': 'no-cache' }
      });
      if (!response.ok) {
        const errorMessage = response.status === 404
          ? `Snapshot endpoint not found: ${this.endpoint}. Make sure the backend service is running.`
          : response.status === 0
            ? `Network error: Unable to connect to ${this.endpoint}. Check if the service is running and CORS is configured.`
            : `Snapshot request failed with status ${response.status}: ${response.statusText}`;
        throw new Error(errorMessage);
      }
      const payload = (await response.json()) as SnapshotPayload;
      this.listeners.forEach((onError, listener) => {
        listener(payload);
      });
    } catch (error) {
      let err: Error;
      if (error instanceof TypeError && error.message.includes('Failed to fetch')) {
        // Network error - provide helpful message
        err = new Error(
          `Cannot connect to ${this.endpoint}. ` +
          `Please ensure the backend service is running. ` +
          `If using a backend API, check that it's accessible at this URL.`
        );
      } else if (error instanceof Error) {
        err = error;
      } else {
        err = new Error('Unknown snapshot error');
      }

      this.listeners.forEach((onError) => {
        if (onError) {
          onError(err);
        }
      });
    }
  }
}

export const snapshotClient = new SnapshotClient();
