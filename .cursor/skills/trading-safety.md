# Skill: Trading safety

**When:** Any change that touches order execution, live trading, configuration, or broker connectivity.

**Rules:**

1. **Paper first:** Use paper trading port (7497) for testing. Never assume production port.
2. **Gate live trading:** Live trading must be behind an explicit configuration flag; do not enable by default.
3. **Validate config:** Validate all configuration before use; reject invalid or missing required fields.
4. **No secrets in code:** Never commit credentials, API keys, or secrets. Do not log sensitive data.
5. **Tests:** All trading logic and risk calculations must have tests (Catch2 for native). Do not skip tests for risk/order code.

**Reference:** .cursorrules "Security & Best Practices", AGENTS.md "Security", docs/TWS_INTEGRATION_STATUS.md for current APIs.
