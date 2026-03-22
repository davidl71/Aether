# Future Improvements

Backlog items deferred from the active plan; documented for later consideration.

---

## PCAP capture migration (Rust)

**Status:** Deferred (removed from Todo2 plan 2026-03-14).

**Description:**

- Create a Rust `pcap_capture` crate with PCAP structures (replacing or wrapping C++ capture logic where it existed).
- Migrate any C++ pcap_capture usage to the Rust crate so packet capture / diagnostics, if needed, live in the Rust workspace.

**Rationale for deferral:** No current dependency on pcap in the active Rust/TUI/backend path. If packet-level debugging or capture is needed later, this can be reintroduced as a tracked task.

**When to revisit:** When adding network diagnostics, capture-based tests, or compliance/audit requirements that need PCAP.

**Previously tracked as:** Todo2 tasks T-1773491604352380000 (create crate), T-1773491528937826000 (migrate C++). Removed from plan; no exarp task IDs kept.

---

## Stale test documentation

**Status:** Informational (no code change required).

Several docs still describe **native C++ tests** (Catch2, `native/tests/`, `ib_box_spread` binary, TWS integration tests). Those tests were removed with the native build. The docs are kept for historical/runbook context but are **obsolete** for current test runs.

**Current test commands:**

- **Rust:** `just test` or `cd agents/backend && cargo test`
- **Python (nautilus):** `just test-python` or `just test-nautilus`
- **TUI E2E:** `just test-tui-e2e`
- **ShellSpec (scripts):** `./scripts/run_tests.sh`
- **Shortcuts:** `./scripts/shortcuts/run_tests.sh` (Rust + optional ShellSpec)

**Docs updated (no longer stale):** The following were updated to remove or replace native references and point to Rust/nautilus/TUI:

- `docs/ TESTING_STRATEGY.md`
- `docs/runbooks/ibkr/IBKR_TESTING_README.md`
- `docs/TEST_COVERAGE_SETUP.md`
- `docs/TWS_CONNECTION_TEST.md`
- `docs/TEST_RUN_RESULTS.md`
- `docs/TEST_COVERAGE_IMPROVEMENT_PLAN.md`
- `docs/INTEGRATION_TESTING.md`
- `docs/NATS_TESTING_GUIDE.md`
- `docs/BREADCRUMB_LOGGING_TRADING_TESTING.md`

Current test commands: `just test`, `just test-python`, `just test-tui-e2e`, `./scripts/run_tests.sh`, `./scripts/shortcuts/run_tests.sh`.
