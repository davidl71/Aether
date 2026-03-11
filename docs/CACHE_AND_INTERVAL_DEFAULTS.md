# Cache and interval defaults – reference and suggestions

This document lists **default and caching periods** for all components that affect latency, freshness, or API load, and suggests **new defaults** where a change would improve balance (responsiveness vs. load vs. freshness).

---

## 1. IB snapshot and Gateway (Python)

| Setting | Current default | Where | Suggested default | Rationale |
|--------|------------------|--------|-------------------|-----------|
| **SNAPSHOT_CACHE_SECONDS** | 3 | `ib_service.py`, env | **3** | Implemented. TUI/Web poll every 0.5–2 s; 3 s gives more cache hits and lower Gateway load. Use 2 if you prefer maximum freshness. |
| **REAUTH_SLEEP_SECONDS** | 0.5 (clamp 0.1–2.0) | `ibkr_portal_client.py`, env | **0.5** | Keep; already configurable. 0.3 is possible if reauth is too slow. |
| **ACCOUNTS_CACHE_TTL_SECONDS** | 2.0 (hardcoded) | `ibkr_portal_client.py` | **2.0** or make env (e.g. align with SNAPSHOT_CACHE) | Keep 2 s; aligns with snapshot build cadence. Optional: expose as env and tie to SNAPSHOT_CACHE_SECONDS. |
| **Portal client timeout_seconds** | 5 | `ibkr_portal_client.py`, `ib_service.py` | **5** | Keep; Gateway responses usually within 1–2 s. |
| **Conid cache** | No TTL (process lifetime) | `ibkr_portal_client.py` | No change | Process-scoped is correct; prewarm reduces cold cost. |

---

## 2. Legacy TUI (Python Textual)

| Setting | Current default | Where | Suggested default | Rationale |
|--------|------------------|--------|-------------------|-----------|
| **update_interval_ms** (provider poll) | 1000 | `config.py`, `shared_config_loader.py`, `config.example.json` | **1000** | Historical Textual TUI setting retained for reference during migration cleanup. |
| **rest_timeout_ms** | 15000 | Same | **15000** | Keep; allows slow Gateway without premature timeout. |
| **ibkr_rest.timeout_ms** | 5000 | `config.example.json`, `config.py` | **5000** | Keep. |
| **refresh_rate_ms** | 500 | `config.example.json` | **500** | Historical Textual TUI refresh rate retained for reference. |
| **_update_snapshot** (set_interval) | 0.5 s | `app.py` (hardcoded) | **0.5** or use config | Keep 0.5 s so UI updates every 500 ms; consider reading from config (e.g. `snapshot_interval_ms`) if we want one knob. |
| **_update_box_spread_data** | 2.0 s | `app.py` (hardcoded) | **2.0** | Keep. |
| **_fetch_bank_accounts** | 30.0 s | `app.py` (hardcoded) | **30.0** | Keep. |
| **_drain_tui_logs** | 0.25 s | `app.py` (hardcoded) | **0.25** | Keep. |

---

## 3. Web (React / PWA)

| Setting | Current default | Where | Suggested default | Rationale |
|--------|------------------|--------|-------------------|-----------|
| **POLL_INTERVAL** (snapshot fallback) | 2000 ms | `web/src/api/snapshot.ts` | **2000** | Matches “Updates every 2 seconds” in UI; keeps load reasonable. |
| **useBankAccounts pollIntervalMs** | 30_000 ms | `web/src/hooks/useBankAccounts.ts` | **30_000** | Keep; bank list does not need sub-minute updates. |
| **PWA update check** | 60 * 60 * 1000 (1 h) | `web/src/hooks/usePWAUpdate.ts` | **1 h** | Keep. |
| **DEFAULT_WS_RECONNECT_INTERVAL** | 3000 ms | `web/src/api/snapshot.ts` | **3000** | Keep. |

---

## 4. Strategy / config (JSON and shared config)

| Setting | Current default | Where | Suggested default | Rationale |
|--------|------------------|--------|-------------------|-----------|
| **loop_delay_ms** | 1000 | `config.example.json` | **1000** | Keep. |
| **connection_timeout_ms** (TWS) | 60000 | `config.example.json` | **60000** | Keep. |
| **reconnect_delay_ms** (TWS) | 3000 | `config.example.json` | **3000** | Keep. |
| **tui.update_interval_ms** | 1000 | `config.example.json` | **1000** | Keep. |
| **tui.rest_timeout_ms** | 15000 | `config.example.json` | **15000** | Keep. |
| **tui.ibkr_rest.timeout_ms** | 5000 | `config.example.json` | **5000** | Keep. |

---

## 5. Third-party / reference data caches

| Setting | Current default | Where | Suggested default | Rationale |
|--------|------------------|--------|-------------------|-----------|
| **ORATS cache_duration_seconds** | 300 (5 min) | `orats_client.py`, config | **300** | Keep; options data acceptable at 5 min. 180 if you want fresher at higher API usage. |
| **Treasury API cache_duration** | 3600 (1 h) | `treasury_api_client.py` | **3600** | Keep; rates are daily. 86400 (24 h) also reasonable. |
| **Massive cache_duration_seconds** | 300 | `config.example.json` | **300** | Keep. |

---

## 6. Summary of suggested changes

- **SNAPSHOT_CACHE_SECONDS default is now 3** (was 2). Implemented in code and docs. All other defaults unchanged.
- **All other defaults:** Keep as-is unless you have a specific need. The Python/Textual TUI entries in this document are historical reference points, not the active terminal runtime defaults.

---

## 7. Where to change defaults

| Component | File(s) |
|-----------|--------|
| IB snapshot cache | `python/integration/ib_service.py` (`_snapshot_cache_ttl_seconds`) |
| IB reauth sleep | `python/integration/ibkr_portal_client.py` (`_reauth_sleep_seconds`) |
| IB accounts cache TTL | `python/integration/ibkr_portal_client.py` (`ACCOUNTS_CACHE_TTL_SECONDS`) |
| Legacy TUI provider interval / timeouts | `python/tui/config.py`, `python/integration/shared_config_loader.py`, `config/config.example.json` |
| Legacy TUI app intervals | `python/tui/app.py` (`set_interval` calls) |
| Web snapshot poll | `web/src/api/snapshot.ts` (`POLL_INTERVAL`) |
| Web bank accounts poll | `web/src/hooks/useBankAccounts.ts` |
| ORATS cache | `python/integration/orats_client.py`, config / strategy_runner |
| Treasury cache | `python/integration/treasury_api_client.py` |
