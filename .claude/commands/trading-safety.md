---
description: Checklist for any change touching order execution, live trading, or broker connectivity
---

Before completing any change that touches orders, live trading, config, or broker connectivity:

1. **Paper first**: Verify paper trading port (7497) is used for testing — never assume production
2. **Gate live trading**: Live trading must be behind an explicit config flag; disabled by default
3. **Validate config**: All configuration validated before use; reject invalid/missing required fields
4. **No secrets in code**: No credentials, API keys, or secrets committed; no sensitive data logged
5. **Tests required**: All trading logic and risk calculations must have Catch2 tests — do not skip
6. **TWS API**: Check `docs/TWS_INTEGRATION_STATUS.md` for current API status before using any TWS calls
7. **Position limits**: Verify position limits are enforced before order submission

Flag any violations as blockers before proceeding.
