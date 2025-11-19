# AI Agent Guide: Shell Script Architecture

This guide provides context for AI agents working with shell scripts in this project.

## Quick Reference

### Shared Include Files Location

All shared functions are in `scripts/include/`:

- `python_utils.sh` - Python virtual environment and package management
- `config.sh` - Configuration loading from config.json
- `service_utils.sh` - Service health checks and port validation
- `onepassword.sh` - 1Password credential management

### Service Scripts Using Shared Functions

All service startup scripts use these shared functions:

- `web/scripts/run-alpaca-service.sh`
- `web/scripts/run-ib-service.sh`
- `web/scripts/run-tradestation-service.sh`
- `web/scripts/run-discount-bank-service.sh`
- `scripts/start_alpaca_service.sh`

## Common Patterns

### Pattern 1: Loading Shared Functions

```bash
ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
SCRIPTS_DIR="${ROOT_DIR}/scripts"

# Load shared utility functions
source "${SCRIPTS_DIR}/include/config.sh"
source "${SCRIPTS_DIR}/include/python_utils.sh"
source "${SCRIPTS_DIR}/include/service_utils.sh"
source "${SCRIPTS_DIR}/include/onepassword.sh"
```

### Pattern 2: Python Setup

```bash
# Find Python
find_python || exit 1

# Set up virtual environment
setup_venv "${PYTHON_DIR}" || exit 1

# Install packages
install_python_packages "${VENV_PYTHON}" "fastapi" "uvicorn[standard]" || exit 1

# Use venv Python
PYTHON_CMD="${VENV_PYTHON}"
```

### Pattern 3: Port Configuration

```bash
# Get port from config (with env var override)
PORT=$(config_get_port "alpaca" 8000)

# Check port availability
if ! check_port_with_service "${PYTHON_CMD}" "127.0.0.1" "${PORT}" "ALPACA_SERVICE" "Alpaca"; then
  exit 1
fi
```

### Pattern 4: Credential Management

```bash
# Read from 1Password or environment variable
API_KEY=$(read_credential "${OP_API_KEY_SECRET}" "${API_KEY:-}" || echo "")

# Check for required credentials
if [ -z "${API_KEY}" ]; then
  echo "Error: API key not set" >&2
  exit 1
fi
```

## Function Reference

### python_utils.sh

| Function | Purpose | Returns |
|----------|---------|---------|
| `find_python()` | Find Python interpreter | Sets `PYTHON_CMD`, returns 0/1 |
| `setup_venv(dir)` | Create/activate venv | Sets `VENV_DIR`, `VENV_PYTHON`, returns 0/1 |
| `install_python_packages(venv, pkg...)` | Install missing packages | Returns 0/1 |
| `test_python_import(venv, module)` | Test module import | Returns 0/1 |
| `disable_init_py(dir)` | Disable __init__.py | Sets `INIT_PY`, `INIT_PY_BAK`, returns 0/1 |

### config.sh

| Function | Purpose | Returns |
|----------|---------|---------|
| `config_get_port(service, default)` | Get service port | Port number or default |
| `config_check_port_available(port)` | Check if port free | 0=available, 1=in use |
| `config_get(path, default)` | Get config value | Value or default |

### service_utils.sh

| Function | Purpose | Returns |
|----------|---------|---------|
| `check_service_health(python, host, port, name)` | Check health endpoint | 0=healthy, 1=unhealthy |
| `check_port_with_service(python, host, port, name, display)` | Check port + verify service | 0=ok, 1=conflict |

### onepassword.sh

| Function | Purpose | Returns |
|----------|---------|---------|
| `read_credential(op_path, env_var)` | Read credential | Credential value or empty |
| `op_detect_fields(uuid, key_var, secret_var)` | Auto-detect fields | Sets variables, returns 0/1 |
| `op_build_secret_paths(uuid, key, secret, key_var, secret_var)` | Build op:// paths | Sets variables, returns 0/1 |

## Global Variables

Functions set these global variables:

- `PYTHON_CMD` - Python interpreter path
- `VENV_DIR` - Virtual environment directory
- `VENV_PYTHON` - Venv Python executable
- `ACTIVATE_PATH` - Venv activation script path
- `INIT_PY` - __init__.py file path
- `INIT_PY_BAK` - Backup __init__.py path
- `SERVICE_HEALTH_CHECK` - Service health check result

## Error Handling

All functions:
- Return `0` on success, `1` on failure
- Write error messages to `stderr`
- Service scripts should check return codes and exit on failure

Example:
```bash
find_python || exit 1
setup_venv "${PYTHON_DIR}" || exit 1
```

## Testing

Tests are in `spec/scripts/include/`:

- `python_utils_spec.sh` - Tests for Python utilities
- `config_spec.sh` - Tests for configuration functions
- `service_utils_spec.sh` - Tests for service utilities
- `onepassword_spec.sh` - Tests for 1Password functions

Run tests:
```bash
./scripts/run_tests.sh
```

## When Modifying Shared Functions

1. **Update Comments**: Add/update AI context comments at function level
2. **Update Tests**: Add tests for new functionality
3. **Check Dependencies**: Verify all service scripts still work
4. **Run Tests**: `./scripts/run_tests.sh`
5. **Test Manually**: Test with at least one service script

## Common Issues

### Issue: Function not found

**Solution**: Ensure script sources the include file:
```bash
source "${SCRIPTS_DIR}/include/python_utils.sh"
```

### Issue: PYTHON_CMD not set

**Solution**: Call `find_python` before using `PYTHON_CMD`:
```bash
find_python || exit 1
# Now PYTHON_CMD is set
```

### Issue: VENV_PYTHON not set

**Solution**: Call `setup_venv` before using `VENV_PYTHON`:
```bash
setup_venv "${PYTHON_DIR}" || exit 1
# Now VENV_PYTHON is set
```

### Issue: Port conflict

**Solution**: Use `check_port_with_service` to verify service identity:
```bash
if ! check_port_with_service "${PYTHON_CMD}" "127.0.0.1" "${PORT}" "SERVICE_NAME" "Display Name"; then
  exit 1
fi
```

## Best Practices for AI Agents

1. **Read Function Comments**: All functions have detailed AI context comments
2. **Follow Patterns**: Use existing service scripts as examples
3. **Check Return Codes**: Always check function return codes
4. **Use Shared Functions**: Don't duplicate code - use shared functions
5. **Test Changes**: Run tests after modifying shared functions
6. **Update Documentation**: Update this guide when adding new patterns

## Related Documentation

- [Shell Script Testing Guide](./SHELL_SCRIPT_TESTING.md) - Testing with ShellSpec
- [PWA Patterns Applicability](./PWA_PATTERNS_APPLICABILITY.md) - Patterns applied to scripts
- [Refactoring Proof of Concept](./REFACTORING_PROOF_OF_CONCEPT.md) - Refactoring example
