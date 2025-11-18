/**
 * leanClient.ts - LEAN REST API Client
 *
 * Client for interacting with the LEAN REST API wrapper.
 * Supports both REST endpoints and WebSocket for real-time updates.
 */

import type { SnapshotPayload } from '../types/snapshot';

export interface LeanClientOptions {
  baseUrl?: string;
  apiKey?: string;
}

export interface StrategyControlRequest {
  confirm: boolean;
}

export interface CancelOrderRequest {
  order_id: string;
  confirm: boolean;
}

export interface ComboOrderRequest {
  symbols: string[];
  quantities: number[];
  limit_price?: number;
  confirm: boolean;
}

export class LeanClient {
  private readonly baseUrl: string;
  private readonly apiKey?: string;

  constructor(options: LeanClientOptions = {}) {
    const env = (import.meta as unknown as { env?: Record<string, unknown> }).env;

    // Get base URL from options, env var, or default
    let envBaseUrl: string | undefined;
    if (typeof env?.VITE_LEAN_API_URL === 'string' && env.VITE_LEAN_API_URL.length > 0) {
      envBaseUrl = env.VITE_LEAN_API_URL;
    }
    this.baseUrl = options.baseUrl ?? envBaseUrl ?? 'http://localhost:8000';

    // Remove trailing slash
    if (this.baseUrl.endsWith('/')) {
      this.baseUrl = this.baseUrl.slice(0, -1);
    }

    this.apiKey = options.apiKey ?? (typeof env?.VITE_LEAN_API_KEY === 'string' ? env.VITE_LEAN_API_KEY : undefined);
  }

  /**
   * Get WebSocket URL for real-time updates.
   */
  getWebSocketUrl(): string {
    const wsProtocol = this.baseUrl.startsWith('https://') ? 'wss://' : 'ws://';
    const wsHost = this.baseUrl.replace(/^https?:\/\//, '');
    return `${wsProtocol}${wsHost}/ws`;
  }

  /**
   * Get health status.
   */
  async getHealth(): Promise<{ status: string; lean_running: boolean; timestamp: string }> {
    const response = await fetch(`${this.baseUrl}/health`, {
      headers: this.getHeaders(),
    });

    if (!response.ok) {
      throw new Error(`Health check failed: ${response.status} ${response.statusText}`);
    }

    return response.json();
  }

  /**
   * Get system snapshot.
   */
  async getSnapshot(): Promise<SnapshotPayload> {
    const response = await fetch(`${this.baseUrl}/api/v1/snapshot`, {
      headers: this.getHeaders(),
    });

    if (!response.ok) {
      if (response.status === 503) {
        throw new Error('LEAN algorithm is not running');
      }
      throw new Error(`Snapshot request failed: ${response.status} ${response.statusText}`);
    }

    return response.json();
  }

  /**
   * Start strategy.
   */
  async startStrategy(request: StrategyControlRequest = { confirm: true }): Promise<void> {
    const response = await fetch(`${this.baseUrl}/api/v1/strategy/start`, {
      method: 'POST',
      headers: {
        ...this.getHeaders(),
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ detail: response.statusText }));
      throw new Error(error.detail || `Strategy start failed: ${response.status}`);
    }
  }

  /**
   * Stop strategy.
   */
  async stopStrategy(request: StrategyControlRequest = { confirm: true }): Promise<void> {
    const response = await fetch(`${this.baseUrl}/api/v1/strategy/stop`, {
      method: 'POST',
      headers: {
        ...this.getHeaders(),
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ detail: response.statusText }));
      throw new Error(error.detail || `Strategy stop failed: ${response.status}`);
    }
  }

  /**
   * Cancel an order.
   */
  async cancelOrder(request: CancelOrderRequest): Promise<void> {
    const response = await fetch(`${this.baseUrl}/api/v1/orders/cancel`, {
      method: 'POST',
      headers: {
        ...this.getHeaders(),
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ detail: response.statusText }));
      throw new Error(error.detail || `Order cancellation failed: ${response.status}`);
    }
  }

  /**
   * Place buy combo order.
   */
  async buyCombo(request: ComboOrderRequest): Promise<void> {
    const response = await fetch(`${this.baseUrl}/api/v1/combos/buy`, {
      method: 'POST',
      headers: {
        ...this.getHeaders(),
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ detail: response.statusText }));
      throw new Error(error.detail || `Combo buy failed: ${response.status}`);
    }
  }

  /**
   * Place sell combo order.
   */
  async sellCombo(request: ComboOrderRequest): Promise<void> {
    const response = await fetch(`${this.baseUrl}/api/v1/combos/sell`, {
      method: 'POST',
      headers: {
        ...this.getHeaders(),
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ detail: response.statusText }));
      throw new Error(error.detail || `Combo sell failed: ${response.status}`);
    }
  }

  private getHeaders(): Record<string, string> {
    const headers: Record<string, string> = {
      'Accept': 'application/json',
    };

    if (this.apiKey) {
      headers['Authorization'] = `Bearer ${this.apiKey}`;
    }

    return headers;
  }
}

// Default instance
export const leanClient = new LeanClient();
