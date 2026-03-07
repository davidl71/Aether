#!/usr/bin/env bash
# Run the NATS → QuestDB market data writer (subscribes to market-data.tick.>, writes ILP).
# Requires Go; runs nats-questdb-bridge.
# Requires: NATS server, QuestDB with ILP enabled (port 9009).
# Optional env: NATS_URL, QUESTDB_ILP_HOST, QUESTDB_ILP_PORT (Go uses QUESTDB_ILP_ADDR=host:port).

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

# Prefer Go bridge (Core NATS subject market-data.tick.>)
export NATS_USE_CORE=1
export NATS_SUBJECT="${NATS_SUBJECT:-market-data.tick.>}"
if [ -n "${QUESTDB_ILP_HOST:-}" ] || [ -n "${QUESTDB_ILP_PORT:-}" ]; then
  export QUESTDB_ILP_ADDR="${QUESTDB_ILP_HOST:-127.0.0.1}:${QUESTDB_ILP_PORT:-9009}"
fi

if command -v go >/dev/null 2>&1 && [ -f "${ROOT_DIR}/agents/go/cmd/nats-questdb-bridge/main.go" ]; then
  cd "${ROOT_DIR}/agents/go" && exec go run ./cmd/nats-questdb-bridge
fi

echo "QuestDB NATS writer requires Go. Install Go and ensure agents/go/cmd/nats-questdb-bridge exists." >&2
exit 1
