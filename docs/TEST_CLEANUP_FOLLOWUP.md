# Test Cleanup Follow-up Tasks

Follow-up from the redundant/obsolete test cleanup (commit that removed duplicate TUI tests, consolidated config tests, and moved Swiftness scripts). Use this for backlog or Todo2.

---

## 1. Documentation fixes

| Task | Detail |
|------|--------|
| **TEST_COVERAGE_SETUP.md** | Update any remaining reference to `python/integration/test_swiftness_integration.py`; document manual Swiftness check: `python scripts/swiftness_integration_manual.py`. |
| **Manual script index** | Optionally add a "Manual / script checks" section in TEST_EXECUTION_PLAN or TEST_COVERAGE_SETUP listing `scripts/swiftness_integration_manual.py` and how to run it. |

---

## 2. Scripts vs tests

| Task | Detail |
|------|--------|
| **test_swiftness_import.py** | No `test_*` or `Test*`; it's a script with `main()` that imports a Swiftness Excel file. Either move to `scripts/` and rename (e.g. `swiftness_import_manual.py`), or leave in place and document in README/doc that it's a manual script, not a pytest test. **Done:** Documented in script docstring as manual; see `docs/TEST_CLEANUP_FOLLOWUP.md`. |
| **run_security_tests.py** | Fallback runner; skip list is documented. No change required unless you want to remove it and rely only on `pytest python/tests/test_security.py`. |

---

## 3. Deduplicate test patterns

| Task | Detail |
|------|--------|
| **Add python/tests/conftest.py** | Centralize path setup (ensure project root or `python/` on `sys.path` once). Remove repeated `sys.path.insert(0, ...)` and `Path(__file__).parent.parent` from individual test files. |
| **Standardize imports** | Choose one: `from python.integration.X` (repo root, PYTHONPATH=python) or `from integration.X` (run from python/). Update tests to one style; conftest makes this consistent. |
| **Shared mock HTTP response** | Add helper in conftest (e.g. `mock_http_response(json_data, status_code=200)`) and use in test_tradier_client, test_tradestation_client, test_alpaca_client, test_tastytrade_client to replace repeated MagicMock blocks. |
| **Optional: shared tmp config helper** | If useful, add `write_json_config(tmp_path, filename, data)` / `read_json_config(path)` in conftest for tests that write then load JSON config (environment_config, config_loader, loan_manager, tui/tests/test_config). |

---

## 4. test_nats_client.py path style

| Task | Detail |
|------|--------|
| **Use Path for path setup** | Replace `os.path.dirname(os.path.dirname(os.path.abspath(__file__)))` with `Path(__file__).resolve().parent.parent` for consistency with other tests. |

---

## Reference

- Cleanup commit: "Remove redundant tests and consolidate TUI config suite"
- Redundant-patterns analysis: same conversation (path setup, mock response, import style, tmp config).
- TUI tests: `python/tui/tests/test_config.py`, `python/tui/tests/test_models.py`.
- Manual Swiftness: `scripts/swiftness_integration_manual.py`.
- LEAN entrypoint: `Main/README.md`, `Main/test_box_spread_basic.py`.
