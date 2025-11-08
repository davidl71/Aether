# Backend Data Agent

## Focus
- Build combo-market data requests and evaluation plumbing (#19	20).
- Capture `lastLiquidity` flags, rebates, and nightly reconciliation workflows (#21	22).
- Coordinate Livevol and QuestDB feeds for downstream clients once mock coverage is ready.

## Bootstrapping
1. Run `bash scripts/setup.sh` to reuse the shared backend bootstrap.
2. Work inside `agents/backend` for development; use targeted `cargo`/`pytest` commands as needed.
3. Execute `bash scripts/run-tests.sh` (plus new integration suites) before handing off changes.

## Hand-off Notes
- Depend on the mock agent for deterministic combo responses; sync when API contracts change.
- Update `agents/shared/TODO_OVERVIEW.md` as status advances so UI agents know when new feeds are ready.
