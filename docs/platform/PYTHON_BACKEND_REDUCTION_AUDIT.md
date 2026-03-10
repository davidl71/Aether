# Python Backend Reduction Audit

**Last updated**: 2026-03-10
**Purpose**: classify the remaining Python backend surface into `keep` vs `retire/migrate`.

## Scope

Reviewed:
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

### Retired Python frontend HTTP surface

Retired:
- `python/services/calculations_api.py`
- `python/services/analytics_api.py`

Reason:
- shared frontend read-model routes were migrated to the Rust API
- the remaining `cash-flow/management` HTTP route had no active web/TUI consumer
- keeping separate Python service ports for dead frontend endpoints only increased operational surface area

Remaining Python cash-management logic:
- `python/integration/cash_flow_portfolio_manager.py`
- `python/tests/test_cash_flow_portfolio_manager.py`

This logic is retained as an in-process module and can be re-exposed later only if a real consumer appears.

## Recommended migration order

1. Keep Python integration modules only where they have active consumers.
2. Re-expose cash-management analysis only if a real UI or automation caller appears.
3. Continue shrinking Python service wrappers in favor of Rust-owned frontend APIs and direct in-process Python modules.

## Non-goals for this pass

- Broker integration APIs are not being removed yet.
- Discount Bank and risk-free-rate logic remain Python-owned for now.
- Health aggregation can be revisited later, but it is not the first reduction target.
