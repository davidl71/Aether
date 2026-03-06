# Shared Configuration Schema Design

**Version:** 1.0.0
**Last Updated:** 2026-01-04
**Status:** Design Document

## Overview

This document designs a unified JSON configuration file format that supports TUI, PWA, and standalone applications with data source selection. The design is based on research findings from T-110 (data source handling patterns) and T-157 (configuration schema design patterns).

## Config file location (home vs project)

- **The app always uses the home config** when no explicit path is set:
  - **Linux/macOS:** `~/.config/ib_box_spread/config.json`
  - **macOS (alternate):** `~/Library/Application Support/ib_box_spread/config.json`
- **Project config** (`config/config.json`, `config/config.example.json`) is for **reference** and is the **source of defaults** to generate the home config:
  - On first run, if the home config does not exist, it is created by copying from the project’s `config.example.json` (or `config.json`), then the app loads from the home path.
  - After that, only the home config is used; project config is not read for normal runs.
- To force a specific file, set **`IB_BOX_SPREAD_CONFIG`** to the path of your JSON config file.

## Design Goals

1. **Unified Format:** Single JSON schema supporting all applications (TUI, PWA, standalone)
2. **Data Source Selection:** Support multiple data source types (Alpaca, IB, TradeStation, static, mock)
3. **Priority/Fallback Logic:** Primary data source with fallback chain
4. **Environment Overrides:** Environment variables override config file values
5. **Backward Compatible:** Maintain compatibility with existing config files
6. **Extensible:** Easy to add new data source types

## Configuration Schema

### Root Structure

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "version": "1.0.0",
  "dataSources": {
    "primary": "alpaca",
    "fallback": ["ib", "mock"],
    "sources": {
      "alpaca": { ... },
      "ib": { ... },
      "tradestation": { ... },
      "mock": { ... },
      "static": { ... }
    }
  },
  "services": {
    "alpaca": { "port": 8000, "url": "http://localhost:8000" },
    "ib": { "port": 8002, "url": "http://localhost:8002" },
    "tradestation": { "port": 8001, "url": "http://localhost:8001" },
    "discountBank": { "port": 8003, "url": "http://localhost:8003" }
  },
  "tui": {
    "providerType": "rest",
    "updateIntervalMs": 1000,
    "restEndpoint": "http://localhost:8080/api/snapshot"
  },
  "pwa": {
    "servicePorts": {
      "alpaca": 8000,
      "ib": 8002,
      "tradestation": 8001,
      "discountBank": 8003
    }
  },
  "broker": {
    "primary": "ALPACA",
    "priorities": ["alpaca", "ib", "mock"]
  }
}
```

## Data Source Configuration

### Data Source Types

#### 1. Alpaca Data Source

```json
{
  "alpaca": {
    "type": "alpaca",
    "enabled": true,
    "apiKeyId": "${ALPACA_API_KEY_ID}",
    "apiSecretKey": "${ALPACA_API_SECRET_KEY}",
    "baseUrl": "https://paper-api.alpaca.markets",
    "dataBaseUrl": "https://data.alpaca.markets",
    "paper": true,
    "timeoutMs": 30000,
    "rateLimitPerMinute": 200
  }
}
```

#### 2. IB (Interactive Brokers) Data Source

```json
{
  "ib": {
    "type": "ib",
    "enabled": true,
    "connectionType": "tws",  // "tws" or "clientPortal"
    "tws": {
      "host": "127.0.0.1",
      "port": 7497,  // 7497 = paper, 7496 = live
      "clientId": 1,
      "paperTrading": true,
      "accountId": "",
      "connectionTimeoutMs": 60000,
      "autoReconnect": true
    },
    "clientPortal": {
      "gatewayUrl": "https://localhost:5001",
      "oauthConsumerKey": "${IB_OAUTH_CONSUMER_KEY}",
      "oauthConsumerSecret": "${IB_OAUTH_CONSUMER_SECRET}",
      "oauthToken": "${IB_OAUTH_TOKEN}",
      "oauthTokenSecret": "${IB_OAUTH_TOKEN_SECRET}",
      "paperTrading": true,
      "verifySsl": false,
      "timeoutMs": 10000
    }
  }
}
```

#### 3. TradeStation Data Source

```json
{
  "tradestation": {
    "type": "tradestation",
    "enabled": true,
    "apiKey": "${TRADESTATION_API_KEY}",
    "apiSecret": "${TRADESTATION_API_SECRET}",
    "baseUrl": "https://api.tradestation.com",
    "paperTrading": true,
    "timeoutMs": 30000
  }
}
```

#### 4. Mock Data Source

```json
{
  "mock": {
    "type": "mock",
    "enabled": true,
    "symbols": ["SPX", "ES50", "NANOS", "XSP"],
    "updateIntervalMs": 1000
  }
}
```

#### 5. Static/File Data Source

```json
{
  "static": {
    "type": "static",
    "enabled": true,
    "filePath": "web/public/data/snapshot.json",
    "watchFile": false,
    "updateIntervalMs": 5000
  }
}
```

## Data Source Selection Logic

### Primary Source

The `dataSources.primary` field specifies the active data source:

```json
{
  "dataSources": {
    "primary": "alpaca"
  }
}
```

### Fallback Chain

The `dataSources.fallback` array specifies fallback sources in order:

```json
{
  "dataSources": {
    "primary": "alpaca",
    "fallback": ["ib", "mock"]
  }
}
```

**Fallback Logic:**

1. Try primary source
2. If primary fails, try first fallback source
3. If first fallback fails, try next fallback source
4. Continue until a source succeeds or all sources are exhausted

### Source Priority

For applications that support multiple simultaneous sources (like PWA), use priority ordering:

```json
{
  "dataSources": {
    "primary": "alpaca",
    "fallback": ["ib", "mock"],
    "priorities": ["alpaca", "ib", "tradestation", "mock"]
  }
}
```

## Application-Specific Configuration

### TUI Configuration

```json
{
  "tui": {
    "providerType": "rest",  // "mock", "rest", "file", "ibkr_rest", "livevol"
    "updateIntervalMs": 1000,
    "refreshRateMs": 500,
    "restEndpoint": "http://localhost:8080/api/snapshot",
    "restTimeoutMs": 10000,
    "restVerifySsl": false,
    "filePath": "web/public/data/snapshot.json",
    "ibkrRest": {
      "baseUrl": "https://localhost:5001/v1/portal",
      "accountId": "",
      "verifySsl": false,
      "timeoutMs": 10000
    },
    "display": {
      "showColors": true,
      "showFooter": true
    }
  }
}
```

**Mapping to Data Sources:**

- `providerType: "rest"` → Uses `services` configuration
- `providerType: "file"` → Uses `static` data source
- `providerType: "ibkr_rest"` → Uses `ib.clientPortal` configuration
- `providerType: "mock"` → Uses `mock` data source

### PWA Configuration

```json
{
  "pwa": {
    "servicePorts": {
      "alpaca": 8000,
      "ib": 8002,
      "tradestation": 8001,
      "discountBank": 8003,
      "riskFreeRate": 8004,
      "tastytrade": 8005,
      "rustBackend": 8080
    },
    "defaultService": "alpaca",
    "serviceUrls": {
      "alpaca": "http://localhost:8000",
      "ib": "http://localhost:8002",
      "tradestation": "http://localhost:8001",
      "discountBank": "http://localhost:8003"
    }
  }
}
```

**Note:** PWA uses build-time environment variables (`VITE_*_PORT`) which override config file values. To use the **same config file** as TUI, set **`VITE_CONFIG_URL`** to the health dashboard config endpoint (e.g. `http://localhost:8011/api/config`). The health dashboard serves `GET /api/config` from the shared home config; PWA can call `loadSharedConfig(VITE_CONFIG_URL)` and use `services` and `broker.priorities` so both UIs see the same backends and ordering.

### Standalone/Backend Configuration

```json
{
  "broker": {
    "primary": "ALPACA",
    "priorities": ["alpaca", "ib", "mock"]
  },
  "marketData": {
    "provider": "polygon",  // "polygon", "mock"
    "pollIntervalMs": 1000,
    "symbols": ["SPY", "QQQ", "IWM"]
  }
}
```

## Environment Variable Overrides

Environment variables override config file values with this priority:

1. **Environment Variables** (highest priority)
2. **Config File Values**
3. **Default Values** (lowest priority)

### Environment Variable Mapping

| Config Field | Environment Variable | Example |
|--------------|---------------------|---------|
| `dataSources.primary` | `DATA_SOURCE_PRIMARY` | `DATA_SOURCE_PRIMARY=alpaca` |
| `tui.providerType` | `TUI_BACKEND` | `TUI_BACKEND=rest` |
| `tui.restEndpoint` | `TUI_API_URL` | `TUI_API_URL=http://localhost:8080/api/snapshot` |
| `services.alpaca.port` | `ALPACA_PORT` | `ALPACA_PORT=8000` |
| `services.ib.port` | `IB_PORT` | `IB_PORT=8002` |
| `alpaca.apiKeyId` | `ALPACA_API_KEY_ID` | `ALPACA_API_KEY_ID=xxx` |
| `alpaca.apiSecretKey` | `ALPACA_API_SECRET_KEY` | `ALPACA_API_SECRET_KEY=yyy` |

## Configuration File Locations

Configuration files are searched in this order:

1. **Explicit Path** (if provided)
2. **Environment Variable** (`IB_BOX_SPREAD_CONFIG`)
3. **Home Config** (`~/.config/ib_box_spread/config.json`)
4. **macOS Application Support** (`~/Library/Application Support/ib_box_spread/config.json`)
5. **System Config** (`/usr/local/etc/ib_box_spread/config.json`)
6. **System Config** (`/etc/ib_box_spread/config.json`)
7. **Project Config** (`config/config.json`)
8. **Project Example** (`config/config.example.json`)

## JSON Schema Validation

**Canonical schema:** `config/schema.json` (project root). All loaders (TUI/PWA shared config, and any CLI using the unified format) should validate config against this schema before or after parsing. Parsing remains language-specific (Python `SharedConfigLoader`, TypeScript/PWA env, C++ `ConfigManager` when reading shared-format files).

### Schema location and usage

- **File:** `config/schema.json`
- **$id:** `https://schemas.ib_box_spread.dev/config/v1.0.0`
- **Draft:** JSON Schema Draft 2020-12

To validate in code: **Python** use `jsonschema.validate(config_dict, schema)` (optional in `SharedConfigLoader.load_config()` when `jsonschema` is available); **TypeScript** use `ajv` with the schema file; **C++** optional json-schema-validator or document schema as single source of truth.

### Schema Definition (reference)

The canonical definition lives in `config/schema.json`. Condensed reference:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://schemas.ib_box_spread.dev/config/v1.0.0",
  "type": "object",
  "properties": {
    "version": { "type": "string", "pattern": "^\\d+\\.\\d+\\.\\d+$" },
    "dataSources": {
      "type": "object",
      "properties": {
        "primary": { "type": "string", "enum": ["alpaca", "ib", "tradestation", "mock", "static"] },
        "fallback": { "type": "array", "items": { "type": "string" } },
        "sources": { "type": "object", "additionalProperties": { "$ref": "#/$defs/dataSource" } }
      },
      "required": ["primary"]
    },
    "services": { "type": "object" },
    "tui": { "$ref": "#/$defs/tuiConfig" },
    "pwa": { "$ref": "#/$defs/pwaConfig" },
    "broker": { "$ref": "#/$defs/brokerConfig" }
  },
  "required": ["version", "dataSources"]
}
```

See `config/schema.json` for full `$defs` (dataSource, tuiConfig, pwaConfig, brokerConfig).

## Migration from Existing Configs

### From TUI Config

**Existing:** `~/.config/ib_box_spread/tui_config.json`

```json
{
  "provider_type": "rest",
  "rest_endpoint": "http://localhost:8080/api/snapshot",
  "update_interval_ms": 1000
}
```

**New:** `~/.config/ib_box_spread/config.json`

```json
{
  "version": "1.0.0",
  "dataSources": {
    "primary": "mock",
    "sources": {
      "mock": { "type": "mock", "enabled": true }
    }
  },
  "tui": {
    "providerType": "rest",
    "restEndpoint": "http://localhost:8080/api/snapshot",
    "updateIntervalMs": 1000
  }
}
```

### From Standalone Config

**Existing:** `config/config.json`

```json
{
  "broker": {
    "primary": "ALPACA",
    "priorities": ["alpaca", "ib", "mock"]
  },
  "alpaca": {
    "api_key_id": "${ALPACA_API_KEY_ID}",
    "api_secret_key": "${ALPACA_API_SECRET_KEY}"
  }
}
```

**New:** `config/config.json`

```json
{
  "version": "1.0.0",
  "dataSources": {
    "primary": "alpaca",
    "fallback": ["ib", "mock"],
    "sources": {
      "alpaca": {
        "type": "alpaca",
        "apiKeyId": "${ALPACA_API_KEY_ID}",
        "apiSecretKey": "${ALPACA_API_SECRET_KEY}"
      }
    }
  },
  "broker": {
    "primary": "ALPACA",
    "priorities": ["alpaca", "ib", "mock"]
  }
}
```

## Implementation Notes

1. **Backward Compatibility:** Support reading existing config formats and migrate to new format
2. **Schema Validation:** Use JSON Schema for validation (Python: jsonschema, TypeScript: ajv, C++: json-schema-validator)
3. **Default Values:** Provide sensible defaults for all fields
4. **Error Handling:** Graceful degradation if config file missing or invalid
5. **Type Safety:** Use Pydantic (Python), Zod (TypeScript), or custom validators (C++)
6. **Environment Variables:** Support `${VAR_NAME}` placeholders for credentials

## Future Extensions

- **Multiple Active Sources:** Support multiple simultaneous data sources (for PWA)
- **Source Health Monitoring:** Track source availability and auto-failover
- **Source Performance Metrics:** Track latency, error rates for source selection
- **Configuration Versioning:** Support config file versioning and migration
- **Remote Configuration:** Support loading config from remote URL (with caching)

## References

- T-110: Research existing configuration patterns and data source handling
- T-157: Research shared configuration schema design patterns
- T-158: Research multi-language configuration loader patterns
- JSON Schema: https://json-schema.org/
- XDG Base Directory: https://specifications.freedesktop.org/basedir-spec/
