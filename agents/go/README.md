# Go agents and tools

Single-binary tools for config validation, NATS collection, QuestDB fanout, and process supervision.

## Tools

| Command | Purpose |
|--------|--------|
| **collection-daemon** | Unified collector (Epic E5): NATS subscribe + sink pipeline (`NATS KV`, QuestDB ILP, logging) + /metrics. See `docs/platform/COLLECTION_DAEMON_PLAN.md`. |
| **config-validator** | Validate shared config JSON (and optional API contract). Used by `scripts/validate_api_contract.sh` when Go is available. |
| **supervisor** | Run multiple services from a JSON config; restart on crash. Use `scripts/run_supervisor.sh` with `config/services.supervisor.json`. |
| **heartbeat-aggregator** | (See cmd/heartbeat-aggregator.) |

## Build and run

From this directory (`agents/go`):

```bash
go build ./cmd/config-validator
go build ./cmd/collection-daemon
go build ./cmd/supervisor
```

From project root, use the scripts so env and paths are correct:

- Config validation: `./scripts/validate_api_contract.sh` (calls Go config-validator when available).
- NATS→QuestDB: `./scripts/run_questdb_nats_writer.sh` (runs `collection-daemon` with `QUESTDB_ILP_ADDR` set).
- Supervisor: `./scripts/run_supervisor.sh` (sets `SUPERVISOR_CONFIG` and `SUPERVISOR_ROOT`).

## Supervisor config

`config/services.supervisor.json` lists services with `name`, `cmd` (array), and `dir` (relative to `SUPERVISOR_ROOT`). Example:

```json
[
  {"name": "nats", "cmd": ["nats-server", "-js", "-DV"], "dir": "."},
  {"name": "ib", "cmd": ["python", "-m", "uvicorn", "ib_service:app", "--host", "0.0.0.0", "--port", "8002"], "dir": "python/services"}
]
```

Run with: `SUPERVISOR_CONFIG=/path/to/services.supervisor.json SUPERVISOR_ROOT=/path/to/repo ./scripts/run_supervisor.sh`
