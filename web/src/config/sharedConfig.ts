/**
 * Shared Configuration Loader (TypeScript)
 *
 * Implements the unified JSON configuration file format for TUI, PWA, and standalone applications.
 * Based on design from docs/research/architecture/SHARED_CONFIGURATION_SCHEMA.md
 *
 * Note: Zod validation can be added later for runtime schema validation.
 * For now, uses TypeScript types for compile-time type safety.
 */

export interface DataSourceConfig {
  type: 'ib' | 'mock' | 'static';
  enabled?: boolean;
  apiKeyId?: string;
  apiSecretKey?: string;
  baseUrl?: string;
  paper?: boolean;
  timeoutMs?: number;
  rateLimitPerMinute?: number;
  // IB-specific fields
  host?: string;
  port?: number;
  clientId?: number;
  connectionType?: 'tws' | 'clientPortal';
  tws?: {
    host?: string;
    port?: number;
    clientId?: number;
    paperTrading?: boolean;
    accountId?: string;
    connectionTimeoutMs?: number;
    autoReconnect?: boolean;
  };
  clientPortal?: {
    gatewayUrl?: string;
    oauthConsumerKey?: string;
    oauthConsumerSecret?: string;
    oauthToken?: string;
    oauthTokenSecret?: string;
    paperTrading?: boolean;
    verifySsl?: boolean;
    timeoutMs?: number;
  };
  // Additional fields
  [key: string]: unknown;
}

export interface DataSourcesConfig {
  primary: string;
  fallback?: string[];
  priorities?: string[];
  sources?: Record<string, DataSourceConfig>;
}

export interface ServiceConfig {
  port?: number;
  url?: string;
  [key: string]: unknown;
}

export interface TUIConfig {
  providerType?: 'mock' | 'rest' | 'file' | 'ibkr_rest' | 'livevol';
  updateIntervalMs?: number;
  refreshRateMs?: number;
  restEndpoint?: string;
  restTimeoutMs?: number;
  restVerifySsl?: boolean;
  filePath?: string;
  ibkrRest?: {
    baseUrl?: string;
    accountId?: string;
    verifySsl?: boolean;
    timeoutMs?: number;
  };
  display?: {
    showColors?: boolean;
    showFooter?: boolean;
  };
}

export interface PWAConfig {
  servicePorts?: Record<string, number>;
  defaultService?: string;
  serviceUrls?: Record<string, string>;
}

export interface BrokerConfig {
  primary?: string;
  priorities?: string[];
}

export interface SharedConfig {
  version?: string;
  dataSources?: DataSourcesConfig;
  services?: Record<string, ServiceConfig>;
  tui?: TUIConfig;
  pwa?: PWAConfig;
  broker?: BrokerConfig;
  [key: string]: unknown; // Allow additional fields for backward compatibility
}

/**
 * Resolve environment variable placeholders in configuration values
 */
function resolveEnvPlaceholders(value: unknown): unknown {
  if (typeof value === 'string' && value.startsWith('${') && value.endsWith('}')) {
    const varName = value.slice(2, -1);
    const envValue = import.meta.env[varName];
    if (envValue !== undefined) {
      return envValue;
    }
    // Fallback to process.env for Node.js environments
    if (typeof process !== 'undefined' && process.env) {
      return process.env[varName];
    }
    console.warn(`Environment variable ${varName} not found, using placeholder`);
    return value;
  }
  if (Array.isArray(value)) {
    return value.map(resolveEnvPlaceholders);
  }
  if (value && typeof value === 'object') {
    const result: Record<string, unknown> = {};
    for (const [key, val] of Object.entries(value)) {
      result[key] = resolveEnvPlaceholders(val);
    }
    return result;
  }
  return value;
}

/**
 * Apply environment variable overrides to configuration
 */
function applyEnvOverrides(config: SharedConfig): SharedConfig {
  const overrides: Partial<SharedConfig> = {};

  // Data source primary
  const dataSourcePrimary = import.meta.env.VITE_DATA_SOURCE_PRIMARY ||
    (typeof process !== 'undefined' ? process.env.DATA_SOURCE_PRIMARY : undefined);
  if (dataSourcePrimary) {
    if (!overrides.dataSources) {
      overrides.dataSources = config.dataSources || {};
    }
    overrides.dataSources.primary = dataSourcePrimary;
  }

  // Service ports (VITE_*_PORT format)
  if (!overrides.services) {
    overrides.services = config.services || {};
  }

  const serviceNames = ['ib', 'discountBank'];
  for (const serviceName of serviceNames) {
    const envVarName = `VITE_${serviceName.toUpperCase().replace('_', '')}_PORT`;
    const envPort = import.meta.env[envVarName] ||
      (typeof process !== 'undefined' ? process.env[envVarName] : undefined);
    if (envPort) {
      const port = parseInt(String(envPort), 10);
      if (!isNaN(port)) {
        if (!overrides.services[serviceName]) {
          overrides.services[serviceName] = {};
        }
        overrides.services[serviceName].port = port;
      }
    }
  }

  // Merge overrides with config
  return {
    ...config,
    ...overrides,
    dataSources: { ...config.dataSources, ...overrides.dataSources },
    services: { ...config.services, ...overrides.services },
  };
}

/**
 * Load configuration from JSON file
 *
 * In browser environments, configuration must be provided at build time
 * or loaded via API endpoint. This function expects the config to be
 * passed as a parameter or loaded via fetch.
 *
 * @param configJson - Configuration JSON object or URL to fetch config from
 * @returns SharedConfig object
 */
export async function loadSharedConfig(
  configJson?: SharedConfig | string
): Promise<SharedConfig> {
  let config: SharedConfig;

  if (typeof configJson === 'string') {
    // Fetch from URL
    const response = await fetch(configJson);
    if (!response.ok) {
      throw new Error(`Failed to load configuration from ${configJson}: ${response.statusText}`);
    }
    config = await response.json();
  } else if (configJson) {
    // Use provided config object
    config = configJson;
  } else {
    // Default configuration
    config = {
      version: '1.0.0',
      dataSources: {
        primary: 'ib',
        fallback: ['mock'],
        sources: {},
      },
      services: {},
    };
  }

  // Resolve environment variable placeholders
  config = resolveEnvPlaceholders(config) as SharedConfig;

  // Apply environment variable overrides
  config = applyEnvOverrides(config);

  return config;
}

/**
 * Load configuration synchronously (for already-loaded config)
 */
export function loadSharedConfigSync(configJson: SharedConfig): SharedConfig {
  // Resolve environment variable placeholders
  const config = resolveEnvPlaceholders(configJson) as SharedConfig;

  // Apply environment variable overrides
  return applyEnvOverrides(config);
}

/**
 * Get primary data source name from configuration
 */
export function getPrimaryDataSource(config: SharedConfig): string {
  return config.dataSources?.primary || 'ib';
}

/**
 * Get fallback data source names from configuration
 */
export function getFallbackDataSources(config: SharedConfig): string[] {
  return config.dataSources?.fallback || [];
}

/**
 * Get configuration for a specific data source
 */
export function getDataSourceConfig(
  config: SharedConfig,
  sourceName: string
): DataSourceConfig | undefined {
  return config.dataSources?.sources?.[sourceName];
}

/**
 * Get service port from configuration with environment variable override
 */
export function getServicePort(
  config: SharedConfig,
  serviceName: string,
  defaultPort?: number
): number {
  // Check environment variable first (Vite format)
  const envVarName = `VITE_${serviceName.toUpperCase().replace('_', '')}_PORT`;
  const envPort = import.meta.env[envVarName] ||
    (typeof process !== 'undefined' ? process.env[envVarName] : undefined);
  if (envPort) {
    const port = parseInt(String(envPort), 10);
    if (!isNaN(port)) {
      return port;
    }
  }

  // Check config file
  const serviceConfig = config.services?.[serviceName];
  if (serviceConfig?.port) {
    return serviceConfig.port;
  }

  // Fall back to default
  if (defaultPort !== undefined) {
    return defaultPort;
  }

  throw new Error(`Port not found for service '${serviceName}' and no default provided`);
}

/**
 * Get service URL from configuration
 */
export function getServiceUrl(
  config: SharedConfig,
  serviceName: string,
  path = ''
): string {
  const port = getServicePort(config, serviceName);
  const baseUrl = `http://localhost:${port}`;
  return path ? `${baseUrl}${path}` : baseUrl;
}
