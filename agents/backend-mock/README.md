# Backend Mock Agent

## Focus
- Finish mock TWS API scenarios, server, and test harness (#16	18).
- Provide reliable fixtures so other agents can develop against deterministic data.
- Surface regression feedback to the master planner when mock coverage gaps appear.

## Bootstrapping
1. Run `bash scripts/setup.sh` to reuse the shared backend bootstrap.
2. Start focused workflows from `agents/backend` with `cargo test` and targeted pytest modules.
3. Execute `bash scripts/run-tests.sh` before handing back changes.

## Hand-off Notes
- Coordinate with the backend data agent once combo quotes or liquidity metrics depend on mock outputs.
- Update `agents/shared/TODO_OVERVIEW.md` as tasks progress to keep alignment.
