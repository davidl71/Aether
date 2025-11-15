#!/usr/bin/env bash

set -euo pipefail

CONFIG_PATH="config/config.json"

usage() {
  cat <<'EOF'
Usage: scripts/set_tws_port.sh <port>

Updates config/config.json so that the TWS connection uses the specified port.
Common values:
  7497 - TWS paper trading
  7496 - TWS live trading
  4002 - IB Gateway paper trading
  4001 - IB Gateway live trading
EOF
  exit 1
}

if [[ $# -ne 1 ]]; then
  usage
fi

PORT="$1"

if ! [[ $PORT =~ ^[0-9]+$ ]]; then
  echo "Port must be numeric (got '$PORT')" >&2
  exit 1
fi

if [[ ! -f $CONFIG_PATH ]]; then
  echo "Config file '$CONFIG_PATH' not found." >&2
  exit 1
fi

python3 - "$PORT" "$CONFIG_PATH" <<'PY'
import json
import sys
from pathlib import Path

if len(sys.argv) != 3:
    sys.exit("Usage: set_tws_port.py <port> <config_path>")

port = int(sys.argv[1])
config_path = Path(sys.argv[2])

raw_text = config_path.read_text()

def strip_comments(text: str) -> str:
    lines = []
    for line in text.splitlines():
        if "//" in line:
            line = line.split("//", 1)[0]
        lines.append(line)
    return "\n".join(lines)

data = json.loads(strip_comments(raw_text))
data.setdefault("tws", {})["port"] = port

config_path.write_text(json.dumps(data, indent=2) + "\n")
PY

echo "Updated TWS port to ${PORT} in ${CONFIG_PATH}"
