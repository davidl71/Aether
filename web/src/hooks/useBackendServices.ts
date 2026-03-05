import { useEffect, useState, useCallback } from 'react';
import { SERVICE_PORTS, getRustBackendUrl, getHealthAggregatedUrl } from '../config/ports';

export type ServiceAttentionType = 'none' | 'authentication' | 'credentials' | 'configuration' | 'error';

export interface BackendServiceStatus {
  name: string;
  port: number;
  healthy: boolean;
  checking: boolean;
  error?: string;
  lastChecked?: Date;
  authenticated?: boolean;
  authUrl?: string;
  attentionRequired?: ServiceAttentionType;
  attentionMessage?: string;
  enabled?: boolean;
  running?: boolean;
  pid?: number;
}

export interface BackendServicesStatus {
  alpaca: BackendServiceStatus;
  tradestation: BackendServiceStatus;
  ib: BackendServiceStatus;
  discountBank: BackendServiceStatus;
  riskFreeRate: BackendServiceStatus;
  tastytrade: BackendServiceStatus;
  tradier: BackendServiceStatus;
  rustBackend: BackendServiceStatus;
}

const SERVICE_CONFIG = {
  alpaca: { name: 'Alpaca', port: SERVICE_PORTS.alpaca, healthPath: '/api/health' },
  tradestation: { name: 'TradeStation', port: SERVICE_PORTS.tradestation, healthPath: '/api/health' },
  ib: { name: 'IB', port: SERVICE_PORTS.ib, healthPath: '/api/health' },
  discountBank: { name: 'Discount Bank', port: SERVICE_PORTS.discountBank, healthPath: '/api/health' },
  riskFreeRate: { name: 'Risk-Free Rate', port: SERVICE_PORTS.riskFreeRate, healthPath: '/api/health' },
  tastytrade: { name: 'Tastytrade', port: SERVICE_PORTS.tastytrade, healthPath: '/api/health' },
  tradier: { name: 'Tradier', port: SERVICE_PORTS.tradier, healthPath: '/api/health' },
  rustBackend: { name: 'Rust Backend', port: SERVICE_PORTS.rustBackend, healthPath: '/health' },
} as const;

// Map unified health dashboard backend id -> PWA SERVICE_CONFIG key
const DASHBOARD_KEY_MAP: Record<string, keyof typeof SERVICE_CONFIG> = {
  ib: 'ib',
  alpaca: 'alpaca',
  tradestation: 'tradestation',
  tastytrade: 'tastytrade',
  discount_bank: 'discountBank',
  analytics: 'riskFreeRate', // analytics includes risk_free_rate; show under Risk-Free Rate for simplicity
};

// Authentication URLs for services that require web-based login
const AUTH_URLS: Record<string, string> = {
  'IB': 'https://localhost:5000', // IB Client Portal Gateway login page
  'Alpaca': 'https://app.alpaca.markets', // Alpaca dashboard (for API key management)
};

function determineAttentionType(
  serviceName: keyof typeof SERVICE_CONFIG,
  healthy: boolean,
  authenticated: boolean | undefined,
  error?: string,
  data?: any
): { attentionRequired: ServiceAttentionType; attentionMessage?: string } {
  if (!healthy) {
    // Service is down - check if it's a configuration issue
    if (error?.includes('credentials') || error?.includes('authentication') || error?.includes('401') || error?.includes('403')) {
      return {
        attentionRequired: 'credentials',
        attentionMessage: 'Missing or invalid credentials',
      };
    }
    if (error?.includes('timeout') || error?.includes('connection')) {
      return {
        attentionRequired: 'configuration',
        attentionMessage: 'Connection issue - check configuration',
      };
    }
    return {
      attentionRequired: 'error',
      attentionMessage: error || 'Service unavailable',
    };
  }

  // Service is healthy, check for other attention needs
  if (authenticated === false) {
    return {
      attentionRequired: 'authentication',
      attentionMessage: 'Authentication required',
    };
  }

  // Check for credential-related errors in response data
  if (data) {
    const errorMsg = data.error?.toLowerCase() || '';
    if (errorMsg.includes('credential') || errorMsg.includes('api key') || errorMsg.includes('secret')) {
      return {
        attentionRequired: 'credentials',
        attentionMessage: 'Credentials configuration needed',
      };
    }
    if (errorMsg.includes('config') || errorMsg.includes('setting')) {
      return {
        attentionRequired: 'configuration',
        attentionMessage: 'Configuration needed',
      };
    }
    // Check for gateway issues (IB specific)
    if (serviceName === 'ib' && errorMsg.includes('gateway')) {
      return {
        attentionRequired: 'configuration',
        attentionMessage: 'IB Gateway not running',
      };
    }
  }

  return { attentionRequired: 'none' };
}

async function checkServiceHealth(
  serviceName: keyof typeof SERVICE_CONFIG
): Promise<{
  healthy: boolean;
  error?: string;
  authenticated?: boolean;
  authUrl?: string;
  attentionRequired?: ServiceAttentionType;
  attentionMessage?: string;
  enabled?: boolean;
  running?: boolean;
  pid?: number;
}> {
  const config = SERVICE_CONFIG[serviceName];

  // First, check service status from backend API (includes enabled/disabled state)
  let enabled: boolean | undefined;
  let running: boolean | undefined;
  let pid: number | undefined;

  try {
    // Map service name to backend API service name
    const apiServiceName = serviceName === 'discountBank' ? 'discount_bank' :
                          serviceName === 'riskFreeRate' ? 'risk_free_rate' :
                          serviceName === 'rustBackend' ? 'rust_backend' :
                          serviceName.toLowerCase();

    // Only check status for non-rust-backend services (rust backend doesn't have service control)
    if (serviceName !== 'rustBackend') {
      const statusUrl = `${getRustBackendUrl()}/api/v1/services/${apiServiceName}/status`;
      try {
        const statusResponse = await fetch(statusUrl, {
          method: 'GET',
          headers: { 'Cache-Control': 'no-cache' },
        });
        if (statusResponse.ok) {
          const statusData = await statusResponse.json();
          enabled = statusData.enabled;
          running = statusData.running;
          pid = statusData.pid;
        }
      } catch (e) {
        // Status API might not be available, ignore
      }
    }
  } catch (e) {
    // Ignore status API errors
  }

  // Then check health endpoint
  const url = `http://localhost:${config.port}${config.healthPath}`;

  try {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 2000); // 2 second timeout

    const response = await fetch(url, {
      method: 'GET',
      signal: controller.signal,
      headers: { 'Cache-Control': 'no-cache' },
    });

    clearTimeout(timeoutId);

    if (response.ok) {
      const data = await response.json().catch(() => ({}));

      // Check for authentication status in response
      let authenticated: boolean | undefined;
      if (serviceName === 'ib' && typeof data.ib_connected === 'boolean') {
        authenticated = data.ib_connected;
      } else if (serviceName === 'alpaca' && typeof data.alpaca_connected === 'boolean') {
        authenticated = data.alpaca_connected;
      }

      const attention = determineAttentionType(serviceName, true, authenticated, undefined, data);

      return {
        healthy: true,
        authenticated,
        authUrl: AUTH_URLS[config.name],
        enabled: enabled ?? true, // Default to enabled if status API unavailable
        running: running ?? true,
        pid,
        ...attention,
      };
    } else {
      const error = `HTTP ${response.status}`;
      const attention = determineAttentionType(serviceName, false, undefined, error);
      return {
        healthy: false,
        error,
        enabled: enabled ?? true,
        running: running ?? false,
        pid,
        ...attention
      };
    }
  } catch (error) {
    let errorMsg: string;
    if (error instanceof Error) {
      if (error.name === 'AbortError') {
        errorMsg = 'Timeout';
      } else {
        errorMsg = error.message;
      }
    } else {
      errorMsg = 'Unknown error';
    }
    const attention = determineAttentionType(serviceName, false, undefined, errorMsg);
    return {
      healthy: false,
      error: errorMsg,
      enabled: enabled ?? true,
      running: running ?? false,
      pid,
      ...attention
    };
  }
}

/** Fetch unified health from dashboard and return status entries for mapped keys */
async function fetchAggregatedHealth(): Promise<Partial<Record<keyof typeof SERVICE_CONFIG, {
  healthy: boolean;
  error?: string;
  authenticated?: boolean;
  authUrl?: string;
  attentionRequired?: ServiceAttentionType;
  attentionMessage?: string;
  enabled?: boolean;
  running?: boolean;
  pid?: number;
}>>> {
  const url = getHealthAggregatedUrl();
  if (!url) return {};
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), 4000);
  try {
    const response = await fetch(url, { method: 'GET', signal: controller.signal, headers: { 'Cache-Control': 'no-cache' } });
    clearTimeout(timeoutId);
    if (!response.ok) return {};
    const data = await response.json().catch(() => ({}));
    const backends = data?.backends;
    if (!backends || typeof backends !== 'object') return {};
    const result: Partial<Record<keyof typeof SERVICE_CONFIG, {
      healthy: boolean;
      error?: string;
      authenticated?: boolean;
      authUrl?: string;
      attentionRequired?: ServiceAttentionType;
      attentionMessage?: string;
      enabled?: boolean;
      running?: boolean;
      pid?: number;
    }>> = {};
    for (const [backendId, payload] of Object.entries(backends)) {
      const pwaKey = DASHBOARD_KEY_MAP[backendId];
      if (!pwaKey || result[pwaKey] !== undefined) continue;
      const p = payload as Record<string, unknown>;
      const status = (p?.status as string) ?? 'unknown';
      const healthy = status === 'ok';
      const config = SERVICE_CONFIG[pwaKey];
      let authenticated: boolean | undefined;
      if (pwaKey === 'ib' && typeof p?.ib_connected === 'boolean') authenticated = p.ib_connected;
      else if (pwaKey === 'alpaca' && typeof p?.alpaca_connected === 'boolean') authenticated = p.alpaca_connected;
      const attention = determineAttentionType(pwaKey, healthy, authenticated, healthy ? undefined : (p?.error as string), p);
      result[pwaKey] = {
        healthy,
        error: healthy ? undefined : ((p?.error as string) ?? `status: ${status}`),
        authenticated,
        authUrl: AUTH_URLS[config.name],
        enabled: true,
        running: true,
        ...attention,
      };
    }
    return result;
  } catch {
    clearTimeout(timeoutId);
    return {};
  }
}

export function useBackendServices(options: { intervalMs?: number; enabled?: boolean } = {}) {
  const { intervalMs = 10000, enabled = true } = options;

  const [statuses, setStatuses] = useState<BackendServicesStatus>(() => {
    const initial: Partial<BackendServicesStatus> = {};
    for (const key of Object.keys(SERVICE_CONFIG) as Array<keyof typeof SERVICE_CONFIG>) {
      initial[key] = {
        name: SERVICE_CONFIG[key].name,
        port: SERVICE_CONFIG[key].port,
        healthy: false,
        checking: false,
      };
    }
    return initial as BackendServicesStatus;
  });

  const checkAllServices = useCallback(async () => {
    if (!enabled) return;

    // Set all services to checking state
    setStatuses((prev) => {
      const updated: Partial<BackendServicesStatus> = {};
      for (const key of Object.keys(SERVICE_CONFIG) as Array<keyof typeof SERVICE_CONFIG>) {
        updated[key] = { ...prev[key], checking: true };
      }
      return updated as BackendServicesStatus;
    });

    const aggregatedUrl = getHealthAggregatedUrl();
    const keysToCheck = Object.keys(SERVICE_CONFIG) as Array<keyof typeof SERVICE_CONFIG>;
    let aggregatedPart: Partial<Record<keyof typeof SERVICE_CONFIG, Awaited<ReturnType<typeof checkServiceHealth>>>> = {};

    if (aggregatedUrl) {
      try {
        const partial = await fetchAggregatedHealth();
        for (const k of keysToCheck) {
          if (partial[k] !== undefined) aggregatedPart[k] = partial[k] as Awaited<ReturnType<typeof checkServiceHealth>>;
        }
      } catch {
        // ignore; will fall back to per-service checks for all
      }
    }

    const checks = Promise.allSettled(
      keysToCheck.map(async (key) => {
        if (aggregatedPart[key] !== undefined) return { key, ...aggregatedPart[key]! };
        return { key, ...(await checkServiceHealth(key)) };
      })
    );

    const results = await checks;

    // Update statuses with results
    setStatuses((prev) => {
      const updated: Partial<BackendServicesStatus> = {};

      // First, process all fulfilled results
      for (const result of results) {
        if (result.status === 'fulfilled') {
          const { key, healthy, error, authenticated, authUrl, attentionRequired, attentionMessage, enabled, running, pid } = result.value;
          updated[key] = {
            ...prev[key],
            healthy,
            checking: false,
            error,
            authenticated,
            authUrl,
            attentionRequired,
            attentionMessage,
            enabled,
            running,
            pid,
            lastChecked: new Date(),
          };
        }
      }

      // Then, handle any rejected promises (shouldn't happen, but handle gracefully)
          const fulfilledKeys = new Set(
        results
          .filter((r): r is PromiseFulfilledResult<{ key: keyof typeof SERVICE_CONFIG; healthy: boolean; error?: string; authenticated?: boolean; authUrl?: string; attentionRequired?: ServiceAttentionType; attentionMessage?: string; enabled?: boolean; running?: boolean; pid?: number }> =>
            r.status === 'fulfilled'
          )
          .map((r) => r.value.key)
      );

      // Mark any services that weren't checked as unhealthy
      for (const key of Object.keys(SERVICE_CONFIG) as Array<keyof typeof SERVICE_CONFIG>) {
        if (!fulfilledKeys.has(key)) {
          updated[key] = {
            ...prev[key],
            healthy: false,
            checking: false,
            error: 'Check failed',
            attentionRequired: 'error',
            attentionMessage: 'Health check failed',
            lastChecked: new Date(),
          };
        }
      }

      return { ...prev, ...updated };
    });
  }, [enabled]);

  useEffect(() => {
    if (!enabled) return;

    // Initial check
    checkAllServices();

    // Set up interval
    const intervalId = setInterval(checkAllServices, intervalMs);

    return () => {
      clearInterval(intervalId);
    };
  }, [checkAllServices, intervalMs, enabled]);

  return {
    statuses,
    checkAllServices,
  };
}
