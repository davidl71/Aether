#!/usr/bin/env bash
# Helper script to inspect 1Password item and show how to use it with Alpaca service
# Usage: ./check-alpaca-1password-item.sh <item-uuid>

set -euo pipefail

if [ $# -eq 0 ]; then
  echo "Usage: $0 <1password-item-uuid>" >&2
  echo "Example: $0 ldfc5jfigtmjvlg6ls4tgpgsuu" >&2
  exit 1
fi

ITEM_UUID="$1"

if ! command -v op >/dev/null 2>&1; then
  echo "Error: 1Password CLI (op) is not installed" >&2
  exit 1
fi

echo "Inspecting 1Password item: ${ITEM_UUID}"
echo ""

# Get item details
ITEM_JSON=$(op item get "${ITEM_UUID}" --format json 2>/dev/null || echo "")

if [ -z "${ITEM_JSON}" ]; then
  echo "Error: Could not retrieve item. Make sure you're signed in: op signin" >&2
  exit 1
fi

# Extract item info
ITEM_TITLE=$(echo "${ITEM_JSON}" | python3 -c "import sys, json; print(json.load(sys.stdin).get('title', 'Unknown'))" 2>/dev/null || echo "Unknown")
VAULT_NAME=$(echo "${ITEM_JSON}" | python3 -c "import sys, json; print(json.load(sys.stdin).get('vault', {}).get('name', 'Unknown'))" 2>/dev/null || echo "Unknown")
VAULT_ID=$(echo "${ITEM_JSON}" | python3 -c "import sys, json; print(json.load(sys.stdin).get('vault', {}).get('id', ''))" 2>/dev/null || echo "")

echo "Item: ${ITEM_TITLE}"
echo "Vault: ${VAULT_NAME}"
echo "UUID: ${ITEM_UUID}"
echo ""

# Extract and display fields
echo "Available fields:"
echo "${ITEM_JSON}" | python3 -c "
import sys, json
data = json.load(sys.stdin)
fields = data.get('fields', [])
for field in fields:
    label = field.get('label', '')
    field_type = field.get('type', '')
    value_preview = field.get('value', '')[:20] + '...' if len(field.get('value', '')) > 20 else field.get('value', '')
    if field_type == 'concealed':
        value_preview = '***' + value_preview[-4:] if len(value_preview) > 4 else '***'
    print(f\"  - {label} ({field_type}): {value_preview}\")
" 2>/dev/null || echo "  (Could not parse fields)"

echo ""
echo "To use this item with the Alpaca service, run:"
echo ""
echo "  export OP_ALPACA_ITEM_UUID='${ITEM_UUID}'"
echo "  ./web/scripts/run-alpaca-service.sh"
echo ""
echo "Or use explicit field references:"
echo ""
if [ -n "${VAULT_ID}" ]; then
  echo "  export OP_ALPACA_API_KEY_ID_SECRET='op://${VAULT_ID}/${ITEM_UUID}/API Key ID'"
  echo "  export OP_ALPACA_API_SECRET_KEY_SECRET='op://${VAULT_ID}/${ITEM_UUID}/API Secret Key'"
else
  echo "  export OP_ALPACA_API_KEY_ID_SECRET='op://${VAULT_NAME}/${ITEM_UUID}/API Key ID'"
  echo "  export OP_ALPACA_API_SECRET_KEY_SECRET='op://${VAULT_NAME}/${ITEM_UUID}/API Secret Key'"
fi
echo "  ./web/scripts/run-alpaca-service.sh"
