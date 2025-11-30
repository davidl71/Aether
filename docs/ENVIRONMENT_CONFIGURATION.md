# Environment Variable Configuration

**Date**: 2025-11-29
**Status**: Implemented

---

## Overview

Centralized environment variable configuration system that provides:

- File-based defaults (`config/environment.json`)
- Environment variable overrides (env vars take precedence)
- Type-safe configuration access
- Security-focused defaults

---

## Configuration File

**Location**: `config/environment.json`

**Structure**:

```json
{
  "security": {
    "rate_limit_per_minute": 60,
    "rate_limit_per_second": 10,
    "api_key": null,
    "require_auth": false,
    "allowed_origins": [...]
  },
  "services": {
    "web_port": 5173,
    "alpaca_port": 8000,
    ...
  }
}
```

---

## Usage

### Basic Usage

```python
from python.services.environment_config import get_config

config = get_config()

# Get security configuration

security = config.get_security_config()
rate_limit = security['rate_limit_per_minute']

# Get specific value

value = config.get('security.rate_limit_per_minute', default=60)
```

### Environment Variable Override

Environment variables take precedence over config file:

```bash

# Override rate limit via environment variable

export RATE_LIMIT_PER_MINUTE=120
python3 app.py  # Uses 120, not config file value
```

### Service Ports

```python
config = get_config()

# Get service port (checks env var first, then config file)

web_port = config.get_service_port('web', default=5173)
alpaca_port = config.get_service_port('alpaca', default=8000)
```

---

## Environment Variables

### Security Variables

| Variable | Config Key | Default | Description |
|----------|------------|---------|-------------|
| `RATE_LIMIT_PER_MINUTE` | `security.rate_limit_per_minute` | 60 | Max requests per minute |
| `RATE_LIMIT_PER_SECOND` | `security.rate_limit_per_second` | 10 | Max requests per second |
| `API_KEY` | `security.api_key` | null | API key for authentication |
| `REQUIRE_AUTH` | `security.require_auth` | false | Require API key auth |

### Service Ports

| Variable | Config Key | Default | Description |
|----------|------------|---------|-------------|
| `WEB_PORT` | `services.web_port` | 5173 | Web service port |
| `ALPACA_PORT` | `services.alpaca_port` | 8000 | Alpaca service port |
| `IB_PORT` | `services.ib_port` | 8002 | IB service port |
| ... | ... | ... | ... |

---

## Integration

### Security Integration Helper

The `security_integration_helper.py` automatically uses environment config:

```python
from python.services.security_integration_helper import add_security_to_app

app = FastAPI()
add_security_to_app(app)  # Uses environment config automatically
```

### Manual Usage

```python
from python.services.environment_config import get_config
from python.services.security import RateLimiter, AccessControl

config = get_config()
security = config.get_security_config()

rate_limiter = RateLimiter(
    requests_per_minute=security['rate_limit_per_minute'],
    requests_per_second=security['rate_limit_per_second']
)

access_control = AccessControl(
    api_key=security['api_key'],
    require_auth=security['require_auth']
)
```

---

## Priority Order

Configuration values are resolved in this order:

1. **Environment Variable** (highest priority)
2. **Config File** (`config/environment.json`)
3. **Default Value** (lowest priority)

---

## Type Conversion

Environment variables are automatically converted to appropriate types:

- **Boolean**: `"true"`, `"1"`, `"yes"`, `"on"` → `True`
- **Integer**: `"60"` → `60`
- **Float**: `"3.14"` → `3.14`
- **List**: `"a,b,c"` → `["a", "b", "c"]`

---

## Reloading Configuration

To reload configuration from file:

```python
from python.services.environment_config import reload_config

reload_config()  # Reloads config file
```

---

## Testing

Security tests verify configuration system:

```bash

# Run security tests

python3 python/tests/run_security_tests.py

# Or with pytest (if available)

pytest python/tests/test_security.py -v
```

---

## Related Documentation

- `python/services/environment_config.py` - Configuration module
- `python/services/security.py` - Security utilities
- `python/services/security_integration_helper.py` - Integration helper
- `python/tests/test_security.py` - Security tests

---

**Last Updated**: 2025-11-29
**Status**: ✅ Implemented and tested
