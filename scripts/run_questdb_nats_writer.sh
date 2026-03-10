#!/usr/bin/env bash
# Run the NATS → QuestDB market data writer through collection-daemon.
# Requires: NATS server, QuestDB with ILP enabled (port 9009), Go.
# Optional env: NATS_URL, QUESTDB_ILP_HOST, QUESTDB_ILP_PORT (Go uses QUESTDB_ILP_ADDR=host:port).

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

if [ -n "${QUESTDB_ILP_HOST:-}" ] || [ -n "${QUESTDB_ILP_PORT:-}" ]; then
  export QUESTDB_ILP_ADDR="${QUESTDB_ILP_HOST:-127.0.0.1}:${QUESTDB_ILP_PORT:-9009}"
fi

export NATS_SUBJECTS="${NATS_SUBJECTS:-market-data.tick.>}"

if command -v go >/dev/null 2>&1 && [ -f "${ROOT_DIR}/agents/go/cmd/collection-daemon/main.go" ]; then
  cd "${ROOT_DIR}/agents/go" && exec go run ./cmd/collection-daemon
fi

echo "QuestDB NATS writer requires Go. Install Go and ensure agents/go/cmd/collection-daemon exists." >&2
exit 1
