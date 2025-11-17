#!/usr/bin/env bash
# Test script to verify Alpaca account retrieval

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
PYTHON_DIR="${PROJECT_ROOT}/python"

cd "${PYTHON_DIR}" || exit 1

# Detect Python command
if command -v python3 >/dev/null 2>&1; then
  PYTHON_CMD="python3"
elif command -v python >/dev/null 2>&1; then
  PYTHON_CMD="python"
else
  echo "Error: Python not found" >&2
  exit 1
fi

echo "Testing Alpaca account retrieval..."
echo ""

"${PYTHON_CMD}" << 'PYTHON_SCRIPT'
import sys
import os
import json

# Add python directory to path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

try:
    from integration.alpaca_client import AlpacaClient

    print("✓ AlpacaClient imported successfully")
    print("")

    # Check environment variables
    api_key = os.getenv("ALPACA_API_KEY_ID", "")
    api_secret = os.getenv("ALPACA_API_SECRET_KEY", "")
    paper_mode = os.getenv("ALPACA_PAPER", "1")
    base_url = os.getenv("ALPACA_BASE_URL", "")

    print("Configuration:")
    print(f"  ALPACA_API_KEY_ID: {'✓ Set' if api_key else '✗ Not set'}")
    print(f"  ALPACA_API_SECRET_KEY: {'✓ Set' if api_secret else '✗ Not set'}")
    print(f"  ALPACA_PAPER: {paper_mode}")
    print(f"  ALPACA_BASE_URL: {base_url or '(using default)'}")
    print("")

    if not api_key or not api_secret:
        print("✗ Error: Alpaca credentials not set")
        print("  Set ALPACA_API_KEY_ID and ALPACA_API_SECRET_KEY environment variables")
        print("  Or use 1Password: export OP_ALPACA_ITEM_UUID=<your-item-uuid>")
        sys.exit(1)

    # Create client
    client = AlpacaClient()
    print(f"✓ AlpacaClient created")
    print(f"  Base URL: {client.base_url}")
    print("")

    # Get accounts
    print("Fetching accounts...")
    accounts = client.get_accounts()

    if not accounts:
        print("✗ No accounts found")
        print("  This might mean:")
        print("  - API keys are incorrect")
        print("  - API endpoint is wrong")
        print("  - Network connection issue")
        sys.exit(1)

    print(f"✓ Found {len(accounts)} account(s):")
    print("")

    for i, acc in enumerate(accounts, 1):
        account_number = acc.get("account_number") or acc.get("id", "N/A")
        account_id = acc.get("id") or acc.get("account_number", "N/A")
        status = acc.get("status", "N/A")
        currency = acc.get("currency", "USD")
        buying_power = acc.get("buying_power", 0.0)
        cash = acc.get("cash", 0.0)
        portfolio_value = acc.get("portfolio_value", 0.0)

        print(f"Account {i}:")
        print(f"  Account Number: {account_number}")
        print(f"  Account ID: {account_id}")
        print(f"  Status: {status}")
        print(f"  Currency: {currency}")
        print(f"  Portfolio Value: ${portfolio_value:,.2f}")
        print(f"  Buying Power: ${buying_power:,.2f}")
        print(f"  Cash: ${cash:,.2f}")
        print("")

        # Check if this is the expected account
        if account_number == "PA3RWI1D1527" or account_id == "PA3RWI1D1527":
            print("  ✓ This matches your paper trading account!")
        print("")

    print("✓ Account retrieval test completed successfully")

except ImportError as e:
    print(f"✗ Import error: {e}")
    print("  Make sure you're in the python directory")
    sys.exit(1)
except Exception as e:
    print(f"✗ Error: {e}")
    import traceback
    traceback.print_exc()
    sys.exit(1)
PYTHON_SCRIPT
