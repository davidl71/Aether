# Python Backend Reduction Audit

**Last updated**: 2026-03-11
**Purpose**: classify the remaining Python specialist-service surface into `keep` vs `retire/migrate`.

## Scope

Reviewed:
- `python/services/health_dashboard.py`
- `python/integration/*_service.py` integration APIs

## Keep in Python

These are still Python-specific or integration-heavy enough that they should remain Python-owned for now.
They are explicit specialist services, not the shared frontend read-model backend and not collection ownership.

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
- Standalone benchmark/rate service wrapper retired.
  - Public route ownership and active implementation now live in the Rust API.
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

Removed as dead internal-only logic:
- `python/integration/cash_flow_portfolio_manager.py`
- `python/tests/test_cash_flow_portfolio_manager.py`

Removed as inactive broker integration:
- `python/integration/tradestation_service.py`
- `python/integration/tradestation_client.py`
- `python/tests/test_tradestation_client.py`

Reason:
- no active web, TUI, automation, or service consumer remained
- keeping an unconsumed analysis module and tests only increased maintenance surface area
- if cash-management analysis becomes a real product feature later, it should be reintroduced behind an explicit consumer and ownership decision

## Recommended migration order

1. Keep Python integration modules only where they have active consumers.
2. Reintroduce cash-management analysis only if a real UI or automation caller appears.
3. Continue shrinking Python service wrappers in favor of Rust-owned frontend APIs and direct in-process Python modules.
4. Do not add new collection, live-state write, or shared read-model ownership to Python services.

## Non-goals for this pass

- Broker integration APIs are not being removed yet.
- Discount Bank public route ownership has moved to Rust; remaining Python parsing helpers are legacy cleanup work.
- Risk-free-rate finance logic has moved into the Rust API; the old standalone Python service wrapper is retired.
- Health aggregation can be revisited later, but it is not the first reduction target.
