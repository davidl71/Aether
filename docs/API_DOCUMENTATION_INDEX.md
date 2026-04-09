# API and integration documentation index

Canonical pointer for external services and first-party integration docs referenced from `AGENTS.md`, editor rules, and automation. Prefer relative paths below (no URL link-check dependency).

## Rust backend (active)

| Area | Doc |
|------|-----|
| Workspace layout, build, test | `docs/MULTI_LANGUAGE_CODEBASE.md`, `AGENTS.md` |
| REST + snapshot surfaces | `agents/backend/services/backend_service`, `docs/` search **snapshot** |
| NATS subjects and commands | `docs/NATS_TOPICS_REGISTRY.md` |
| NATS health checks | `docs/NATS_HEALTH_VERIFICATION.md` |
| Market data provider model | `docs/MARKET_DATA_PROVIDER_ARCHITECTURE.md` |
| Broker abstraction | `docs/ARCHITECTURE_BROKER_ENGINE.md` |
| TWS / IBKR (Rust path) | `docs/TWS_RUST_RUNBOOK.md`, `docs/TWS_BACKEND_PROVIDER_DECISION.md` |

## IBKR / TWS (vendor)

| Resource | Notes |
|----------|--------|
| TWS API sibling repo | Clone `../tws-api/` next to this repo (see `AGENTS.md`) |
| Paper port | `7497` for testing; gate live trading behind config |

## Protocol and messages

| Area | Doc |
|------|-----|
| Protobuf definitions | `proto/messages.proto`; Rust via `nats_adapter` build |

## exarp-go / MCP (sibling tooling)

| Area | Doc |
|------|-----|
| 3270 TUI | `exarp-go/docs/3270_TUI_IMPLEMENTATION.md` (when using sibling checkout) |
| Go + ISPF research | `exarp-go/docs/GO_3270_ISPF_GITHUB_CGO_RESEARCH.md` |

## Maintenance

- When adding a new external API integration, add a row under the right section and extend `docs/API_DOCUMENTATION_SUMMARY.md` if needed.
- Generated tables: `scripts/generate_docs_summary_tables.py` reads this file when present.
