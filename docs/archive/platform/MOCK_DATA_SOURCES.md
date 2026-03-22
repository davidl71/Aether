# Mock data sources

All backend data can run from **mock sources** so you can develop and test without API keys, live brokers, or external services.

## What has mocks

| Data / API | Real source | Mock source | When mock is used |
|------------|-------------|-------------|-------------------|
| **Snapshot** (positions, orders, historic, symbols, alerts) | IB/TWS, market data loop | `api::mock_data::seed_snapshot` | Backend seeds empty snapshot at startup when using mock market data provider |
| **Market data** (ticks, candles) | Polygon, FMP, or other provider | `market_data::MockMarketDataSource` | When `provider = "mock"` in backend config (default when provider unknown) |
| **Finance rates** (SOFR, Treasury) | FRED API | `api::mock_data::mock_sofr_benchmarks`, `mock_treasury_benchmarks` | When `FRED_API_KEY` is unset or FRED request fails |
| **FMP** (income statement, balance sheet, cash flow, quote) | FMP API | `api::mock_data::mock_fmp_*` | When `FMP_API_KEY` is unset; backend still subscribes to `api.fmp.*` and replies with mock |
| **Discount Bank** (balance, transactions, bank accounts) | File / scraper | `api::mock_data::mock_discount_bank_*` | When file missing or fetch fails |
| **Loans** (list, get, create, update, delete) | LoanRepository (e.g. SQLite) | `api::mock_data::mock_loans_list` | When LoanRepository not configured |
| **Calculate** (greeks, IV, risk, box spread, etc.) | In-process quant/risk crates | N/A (no external source) | Always real calculation; inputs can come from mock snapshot/FMP if needed |

Mock types and helpers live in **`agents/backend/crates/api/src/mock_data.rs`**. Market data mock lives in **`agents/backend/crates/market_data/src/mock.rs`**.

## All-mock mode (no API keys)

To run backend + TUI with **all data from mocks**:

1. **Do not set** `FRED_API_KEY`, `FMP_API_KEY`, or Polygon/FMP provider API keys.
2. **Use mock market data:** In backend config (e.g. `config/default.toml` or env), set market data `provider = "mock"` (or omit/use an unknown provider; backend falls back to mock).
3. **Optional:** Do not configure `LoanRepository` (no DB file); loans API will return mock list.
4. **Optional:** Do not configure Discount Bank file; discount bank NATS handlers will return mock balance/transactions/accounts.

Example (no env vars required):

```bash
cd agents/backend
cargo run -p backend_service
# In another terminal:
cargo run -p tui_service
```

Snapshot will be seeded with mock positions/orders/symbols; finance rates and FMP will reply with mock data; market data will be mock ticks. CLI can test FRED mock via:

```bash
cargo run -p cli -- benchmarks
# No FRED_API_KEY → mock SOFR/Treasury
```

## Env / config summary

| To use mock for | Do this |
|-----------------|--------|
| Finance rates (SOFR, Treasury) | Leave `FRED_API_KEY` unset |
| FMP fundamentals | Leave `FMP_API_KEY` unset |
| Market data stream | Set `provider = "mock"` in backend market data config (or use unknown provider) |
| Snapshot seeding | Use mock market data provider; snapshot is seeded with mock when empty |
| Loans | Do not configure LoanRepository (no DB); handlers return mock list |
| Discount Bank | Do not configure bank file; handlers return mock balance/transactions/accounts |

## References

- NATS API subjects: [NATS_API.md](NATS_API.md)
- Backend config: `config/default.toml`, env `BACKEND_CONFIG`
- Keys when using real data: [KEYS_FROM_1PASSWORD.md](KEYS_FROM_1PASSWORD.md), [BACKEND_SECRETS_PROVIDERS.md](../BACKEND_SECRETS_PROVIDERS.md)
