#!/usr/bin/env bash
# Run TradeStation service for PWA integration
# Supports 1Password integration for secure credential management
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
PYTHON_DIR="$ROOT_DIR/python"

# Function to read from 1Password or environment variable
read_credential() {
  local op_secret="${1:-}"
  local env_var="${2:-}"
  local description="${3:-credential}"

  # Try 1Password first if secret path is provided
  if [ -n "${op_secret:-}" ] && command -v op >/dev/null 2>&1; then
    if op read "${op_secret}" 2>/dev/null | grep -q .; then
      op read "${op_secret}" 2>/dev/null | tr -d '[:space:]'
      return 0
    fi
  fi

  # Fall back to environment variable
  if [ -n "${env_var:-}" ]; then
    echo -n "${env_var}"
    return 0
  fi

  return 1
}

# Read credentials from 1Password or environment variables
OP_CLIENT_ID_SECRET="${OP_TRADESTATION_CLIENT_ID_SECRET:-}"
OP_CLIENT_SECRET_SECRET="${OP_TRADESTATION_CLIENT_SECRET_SECRET:-}"

TRADESTATION_CLIENT_ID=$(read_credential "${OP_CLIENT_ID_SECRET}" "${TRADESTATION_CLIENT_ID:-}" "Client ID" || echo "")
TRADESTATION_CLIENT_SECRET=$(read_credential "${OP_CLIENT_SECRET_SECRET}" "${TRADESTATION_CLIENT_SECRET:-}" "Client Secret" || echo "")

# Check for required credentials
if [ -z "${TRADESTATION_CLIENT_ID}" ] || [ -z "${TRADESTATION_CLIENT_SECRET}" ]; then
  echo "Error: TradeStation credentials not set" >&2
  echo "" >&2
  echo "Option 1: Use 1Password (recommended):" >&2
  echo "  export OP_TRADESTATION_CLIENT_ID_SECRET='op://Vault/Item/Client ID'" >&2
  echo "  export OP_TRADESTATION_CLIENT_SECRET_SECRET='op://Vault/Item/Client Secret'" >&2
  echo "" >&2
  echo "Option 2: Use environment variables:" >&2
  echo "  export TRADESTATION_CLIENT_ID=your_client_id" >&2
  echo "  export TRADESTATION_CLIENT_SECRET=your_client_secret" >&2
  echo "" >&2
  echo "Optional:" >&2
  echo "  export TRADESTATION_SIM=1  # Use SIM environment (default)" >&2
  echo "  export TRADESTATION_ACCOUNT_ID=your_account_id  # Optional account ID" >&2
  echo "  export SYMBOLS=SPY,QQQ,IWM  # Comma-separated symbols (default: SPY,QQQ)" >&2
  exit 1
fi

# Export credentials for the Python service
export TRADESTATION_CLIENT_ID
export TRADESTATION_CLIENT_SECRET

cd "$PYTHON_DIR"

# Check if uvicorn is installed
if ! python -c "import uvicorn" 2>/dev/null; then
  echo "Installing uvicorn..." >&2
  pip install uvicorn fastapi >&2
fi

# Check if integration module is available
if ! python -c "from integration.tradestation_service import app" 2>/dev/null; then
  echo "Error: Cannot import tradestation_service. Make sure you're in the python directory." >&2
  exit 1
fi

echo "Starting TradeStation service on http://127.0.0.1:8001" >&2
echo "Set VITE_API_URL=http://127.0.0.1:8001/api/snapshot in your web app" >&2
echo "" >&2

# Run the service (using port 8001 to avoid conflict with Alpaca on 8000)
python -m uvicorn integration.tradestation_service:app --host 127.0.0.1 --port 8001 --reload
