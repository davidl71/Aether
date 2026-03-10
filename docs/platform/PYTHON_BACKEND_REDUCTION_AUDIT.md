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

Migrate first:
- `POST /api/v1/frontend/unified-positions`
  - thin wrapper around `frontend_views.normalize_bank_accounts_to_positions`
- `POST /api/v1/frontend/relationships`
  - thin wrapper around `frontend_views.infer_relationships`

Migrate after frontend parity:
- `POST /api/v1/cash-flow/timeline`
- `POST /api/v1/cash-flow/management`
- `POST /api/v1/opportunity-simulation/scenarios`
- `POST /api/v1/opportunity-simulation/calculate`

Reason:
- these are shared frontend read models or calculations already consumed as UI-facing API contracts
- Rust should become the shared frontend API owner for both web and TUI

### `python/services/analytics_api.py`

Retire after the underlying calculation and benchmark routes move:
- this file is primarily a router-composition shell over calculations + risk-free-rate
- once Rust owns the frontend-facing read models, this should shrink or disappear

## Recommended migration order

1. Move `frontend/unified-positions` to Rust.
2. Move `frontend/relationships` to Rust.
3. Move cash-flow read models to Rust.
4. Move opportunity simulation read-model API ownership to Rust.
5. Collapse or delete `analytics_api.py` once those routes are no longer Python-fronted.

## Non-goals for this pass

- Broker integration APIs are not being removed yet.
- Discount Bank and risk-free-rate logic remain Python-owned for now.
- Health aggregation can be revisited later, but it is not the first reduction target.
