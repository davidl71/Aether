# Python Backend Reduction Audit

**Last updated**: 2026-03-10
**Purpose**: classify the remaining Python backend surface into `keep` vs `retire/migrate`.

## Scope

Reviewed:
- `python/services/analytics_api.py`
- `python/services/calculations_api.py`
- `python/services/health_dashboard.py`
- `python/integration/*_service.py` integration APIs

## Keep in Python

These are still Python-specific or integration-heavy enough that they should remain Python-owned for now.

### TUI and integration services

- `python/tui/*`
  - Active Python/Textual terminal client.
- `python/integration/discount_bank_service.py`
  - Bank import and ledger extraction logic.
- `python/integration/ib_service.py`
  - IB-specific integration API and session state.
- `python/integration/alpaca_service.py`
  - Broker-specific integration API and session state.
- `python/integration/tastytrade_service.py`
  - Broker-specific integration API and session state.
- `python/integration/tradestation_service.py`
  - Broker-specific integration API and session state.
- `python/integration/risk_free_rate_service.py`
  - FRED / benchmark and curve logic.
- `python/services/health_dashboard.py`
  - NATS-fed health aggregation is still Python-owned today, though it could move later.

## Retire or migrate behind Rust

These endpoints are mostly frontend read-model shaping and should move to Rust-owned API contracts.

### `python/services/calculations_api.py`

Migrated to Rust:
- `POST /api/v1/frontend/unified-positions`
- `POST /api/v1/frontend/relationships`
- `POST /api/v1/cash-flow/timeline`
- `POST /api/v1/opportunity-simulation/scenarios`
- `POST /api/v1/opportunity-simulation/calculate`

Still Python-owned:
- `POST /api/v1/cash-flow/management`

Reason:
- the migrated routes were shared frontend read models or frontend-facing calculations
- Rust should become the shared frontend API owner for both web and TUI

### `python/services/analytics_api.py`

Shrink further after the remaining Python-only route is reviewed:
- this file is now mostly a router-composition shell over `cash-flow/management` and risk-free-rate
- once `cash-flow/management` is either migrated or confirmed as Python-specific, this can shrink again or disappear

## Recommended migration order

1. Review whether `cash-flow/management` is genuinely Python-specific or just another read model.
2. If it is a shared frontend contract, move it to Rust.
3. Collapse or delete `analytics_api.py` once only risk-free-rate remains Python-fronted.

## Non-goals for this pass

- Broker integration APIs are not being removed yet.
- Discount Bank and risk-free-rate logic remain Python-owned for now.
- Health aggregation can be revisited later, but it is not the first reduction target.
