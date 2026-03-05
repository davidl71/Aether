/**
 * Port configuration for backend services
 * Reads from Vite environment variables (VITE_*), with fallbacks to defaults
 */

const getEnvVar = (name: string, defaultValue: number): number => {
  const env = (import.meta as unknown as { env?: Record<string, unknown> }).env;
  const value = env?.[name];
  if (typeof value === 'string') {
    const parsed = parseInt(value, 10);
    if (!isNaN(parsed)) {
      return parsed;
    }
  }
  return defaultValue;
};

/**
 * Service port configuration
 * Ports are read from Vite environment variables (set by run-web-service.sh)
 * or fall back to default values
 */
export const SERVICE_PORTS = {
  alpaca: getEnvVar('VITE_ALPACA_PORT', 8000),
  ib: getEnvVar('VITE_IB_PORT', 8002),
  tradestation: getEnvVar('VITE_TRADESTATION_PORT', 8001),
  discountBank: getEnvVar('VITE_DISCOUNT_BANK_PORT', 8003),
  riskFreeRate: getEnvVar('VITE_RISK_FREE_RATE_PORT', 8004),
  tastytrade: getEnvVar('VITE_TASTYTRADE_PORT', 8005),
  tradier: getEnvVar('VITE_TRADIER_PORT', 8006),
  rustBackend: getEnvVar('VITE_RUST_BACKEND_REST_PORT', 8080),
} as const;

/**
 * Get service URL for a given service
 */
export function getServiceUrl(service: keyof typeof SERVICE_PORTS, path = ''): string {
  const port = SERVICE_PORTS[service];
  const baseUrl = `http://localhost:${port}`;
  return path ? `${baseUrl}${path}` : baseUrl;
}

/**
 * Get Rust backend API URL
 */
export function getRustBackendUrl(path = ''): string {
  return getServiceUrl('rustBackend', path);
}

/** Base URL for API when using nginx/shared server (e.g. http://localhost:8080) */
function getApiBaseUrl(): string {
  const env = (import.meta as unknown as { env?: Record<string, unknown> }).env;
  const apiUrl = env?.VITE_API_URL;
  if (typeof apiUrl === 'string' && apiUrl) {
    const u = apiUrl.trim().replace(/\/api\/?$/, '');
    return u || 'http://localhost:8080';
  }
  return 'http://localhost:8080';
}

/**
 * URL for unified health JSON (dashboard). When set, PWA uses one request instead of per-service.
 * Set VITE_HEALTH_AGGREGATED_URL or use nginx at 8080 with /api/health-aggregated.
 */
export function getHealthAggregatedUrl(): string | null {
  const env = (import.meta as unknown as { env?: Record<string, unknown> }).env;
  const explicit = env?.VITE_HEALTH_AGGREGATED_URL;
  if (typeof explicit === 'string' && explicit.trim()) return explicit.trim();
  const base = getApiBaseUrl();
  return `${base}/api/health-aggregated`;
}
