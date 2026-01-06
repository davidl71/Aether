## Universal Brokerage Aggregation and Embedded Investing Options

This page summarizes options to connect to multiple brokers via a single API and contrasts them with embedded-investing providers. Use this as a
quick KB when deciding between aggregation vs. operating under a single provider license.

### Universal Aggregators (link existing brokerage accounts)

- SnapTrade — multi-broker read/write, near real-time; single schema for accounts, balances, positions, orders.
  Supports connection portal per user (`userId`/`userSecret`) and checked-order flow before placement.
  Recommended when you need to aggregate and trade across users’ existing brokers.
  - Docs: <https://docs.snaptrade.com/docs/getting-started>
  - CLI (quick testing across brokers): <https://github.com/passiv/snaptrade-cli>
- Read-only alternatives — Plaid, Sophtron: broad coverage but no trading. Good for portfolio aggregation/analytics without execution.
  - Overview and tradeoffs: <https://konfigthis.com/blog/asset-management-integrations/>

### Embedded Investing (operate under provider’s license; not an aggregator)

- Finax (EU Investment‑as‑a‑Service) — licensing, KYC/AML, custody (KBC), compliance/taxes, portfolio management, white‑label UI.
  Users invest under Finax’s stack rather than linking external brokers. Appropriate when launching EU investing without becoming a broker.
  - Finax B2B: <https://www.finax.eu/en/b2b-solutions>

### Recommended patterns for this repository

- Keep IBKR native for options/box‑spread execution and market data.
- Add SnapTrade for multi‑broker aggregation and optional trading outside IBKR.
- If targeting EU embedded-investing product lines (new accounts under provider), evaluate Finax separately.

### Minimal operational notes

- Credentials: store `SNAPTRADE_CLIENT_ID`, `SNAPTRADE_CONSUMER_KEY` in environment/secret manager; persist per‑user `userSecret` securely.
- Onboarding flow (SnapTrade):
  create user → generate Connection Portal URL → user links broker(s) → list accounts → fetch balances/positions/orders → (optionally) check order impact → place checked order.

### References

- SnapTrade Getting Started: <https://docs.snaptrade.com/docs/getting-started>
- SnapTrade CLI: <https://github.com/passiv/snaptrade-cli>
- Universal brokerage APIs (Plaid, SnapTrade, Sophtron, etc.): <https://konfigthis.com/blog/asset-management-integrations/>
- Finax Investment‑as‑a‑Service (EU): <https://www.finax.eu/en/b2b-solutions>
