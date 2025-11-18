import type { SnapshotPayload } from '../types/snapshot';

const DEFAULT_ENDPOINT = '/data/snapshot.json';
const POLL_INTERVAL = 2000;

export interface SnapshotClientOptions {
  endpoint?: string;
  pollIntervalMs?: number;
  useLeanApi?: boolean; // Use LEAN API wrapper instead of Rust backend
}

export type SnapshotListener = (payload: SnapshotPayload) => void;
export type SnapshotErrorListener = (error: Error) => void;

export class SnapshotClient {
  private readonly endpoint: string;
  private readonly pollInterval: number;
  private readonly useLeanApi: boolean;
  private timer: number | null = null;
  private listeners: Map<SnapshotListener, SnapshotErrorListener | undefined> =
    new Map<SnapshotListener, SnapshotErrorListener | undefined>();

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
    } else {
      // Use Rust backend endpoint
      let envEndpoint: string | undefined;
      if (typeof env?.VITE_API_URL === 'string' && env.VITE_API_URL.length > 0) {
        envEndpoint = env.VITE_API_URL;
      }
      this.endpoint = options.endpoint ?? envEndpoint ?? DEFAULT_ENDPOINT;
    }

    this.pollInterval = options.pollIntervalMs ?? POLL_INTERVAL;
  }

  subscribe(listener: SnapshotListener, onError?: SnapshotErrorListener) {
    this.listeners.set(listener, onError);
    if (this.listeners.size === 1) {
      void this.fetchOnce();
      this.start();
    }
    return () => this.unsubscribe(listener);
  }

  unsubscribe(listener: SnapshotListener) {
    this.listeners.delete(listener);
    if (this.listeners.size === 0) {
      this.stop();
    }
  }

  private start() {
    this.stop();
    this.timer = window.setInterval(() => {
      void this.fetchOnce();
    }, this.pollInterval);
  }

  private stop() {
    if (this.timer !== null) {
      clearInterval(this.timer);
      this.timer = null;
    }
  }

  private async fetchOnce() {
    try {
      const response = await fetch(this.endpoint, {
        headers: { 'cache-control': 'no-cache' }
      });
      if (!response.ok) {
        throw new Error(`Snapshot request failed with status ${response.status}`);
      }
      const payload = (await response.json()) as SnapshotPayload;
      this.listeners.forEach((onError, listener) => {
        listener(payload);
      });
    } catch (error) {
      const err = error instanceof Error ? error : new Error('Unknown snapshot error');
      this.listeners.forEach((onError) => {
        if (onError) {
          onError(err);
        }
      });
    }
  }
}

export const snapshotClient = new SnapshotClient();
