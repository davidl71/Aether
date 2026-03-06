# Israeli Bank Scrapers Service

Runs [israeli-bank-scrapers](https://github.com/eshaham/israeli-bank-scrapers) on-demand (or via cron), maps results to the shared ledger, and exposes the same data through the existing **Discount Bank service** `GET /api/bank-accounts` (TUI and Web read from the ledger).

## Prerequisites

- **Node.js >= 22.12.0**
- **Chromium** — Puppeteer (used by israeli-bank-scrapers) will download Chromium on first `npm install`. If your environment blocks the download (e.g. corporate proxy/SSL), set `PUPPETEER_SKIP_DOWNLOAD=1` and point the scraper to a system Chromium via the library’s `executablePath` option (see israeli-bank-scrapers docs).

## Install

From project root:

```bash
cd services/israeli-bank-scrapers-service && npm install
```

## Configuration

Credentials are passed via **environment variables** only (never in request body in production).
You can also use **1Password**: set `OP_SCRAPER_*_SECRET` to `op://` paths and run via `./scripts/run_israeli_bank_scrapers_service.sh` (or start the service with the same script); see [1Password](#1password) below.

| Env | Description |
|-----|-------------|
| `LEDGER_DATABASE_PATH` | Ledger SQLite path (default: `ledger.db` in project root or `agents/backend/ledger.db`) |
| `PORT` | HTTP server port (default: `8010`) |
| `SCRAPER_COMPANY_ID` | Company to scrape: `fibi`, `max`, `visaCal`, `discount`, `leumi`, `hapoalim`, `isracard`, `beinleumi`, `mizrahi`, etc. |
| `SCRAPER_START_DATE` | Start date `YYYY-MM-DD` (default: 1 year ago) |

### 1Password

From project root, run the service or CLI via the wrapper so credentials are read from 1Password:

```bash
# 1Password secret references (op://Vault/Item/Field)
export OP_SCRAPER_DISCOUNT_ID_SECRET="op://Vault/My Item/Identification Number"
export OP_SCRAPER_DISCOUNT_PASSWORD_SECRET="op://Vault/My Item/password"
export OP_SCRAPER_DISCOUNT_NUM_SECRET="op://Vault/My Item/code"

# Start HTTP server (port 8010)
./scripts/run_israeli_bank_scrapers_service.sh

# Or run one-off scrape and write to ledger
./scripts/run_israeli_bank_scrapers_service.sh scrape
```

Supported 1Password env vars (fallback to plain `SCRAPER_*` env if unset):

- **Discount**: `OP_SCRAPER_DISCOUNT_ID_SECRET`, `OP_SCRAPER_DISCOUNT_PASSWORD_SECRET`, `OP_SCRAPER_DISCOUNT_NUM_SECRET`
- **Leumi**: `OP_SCRAPER_LEUMI_USERNAME_SECRET`, `OP_SCRAPER_LEUMI_PASSWORD_SECRET`
- **Hapoalim**: `OP_SCRAPER_HAPOALIM_USER_CODE_SECRET`, `OP_SCRAPER_HAPOALIM_PASSWORD_SECRET`
- **Visa Cal**: `OP_SCRAPER_VISACAL_USERNAME_SECRET`, `OP_SCRAPER_VISACAL_PASSWORD_SECRET`
- **Isracard**: `OP_SCRAPER_ISRACARD_ID_SECRET`, `OP_SCRAPER_ISRACARD_CARD_6_DIGITS_SECRET`, `OP_SCRAPER_ISRACARD_PASSWORD_SECRET`
- **Max**: `OP_SCRAPER_MAX_USERNAME_SECRET`, `OP_SCRAPER_MAX_PASSWORD_SECRET`
- **Beinleumi**: `OP_SCRAPER_BEINLEUMI_USERNAME_SECRET`, `OP_SCRAPER_BEINLEUMI_PASSWORD_SECRET`
- **Generic**: `OP_SCRAPER_USERNAME_SECRET`, `OP_SCRAPER_PASSWORD_SECRET`, `OP_SCRAPER_ID_SECRET`, `OP_SCRAPER_NUM_SECRET`, `OP_SCRAPER_CARD_6_DIGITS_SECRET`

**Auth (choose one):**

- **1Password CLI**: `op signin` (wrapper script uses `op read`).
- **1Password SDK (formal API)**: Set `OP_SERVICE_ACCOUNT_TOKEN` or `OP_1PASSWORD_ACCOUNT_NAME` (desktop app). Then run the Node server or CLI directly (e.g. `npm start`, `npm run scrape`); secrets in `OP_SCRAPER_*_SECRET` are resolved in-process via [@1password/sdk](https://github.com/1Password/onepassword-sdk-js). No shell wrapper required.

See `docs/ONEPASSWORD_INTEGRATION.md`.

### Credentials per company (env or 1Password)

- **Discount**: `SCRAPER_DISCOUNT_ID`, `SCRAPER_DISCOUNT_PASSWORD`, `SCRAPER_DISCOUNT_NUM` (or generic `SCRAPER_ID`, `SCRAPER_PASSWORD`, `SCRAPER_NUM`)
- **Leumi**: `SCRAPER_LEUMI_USERNAME`, `SCRAPER_LEUMI_PASSWORD`
- **Hapoalim**: `SCRAPER_HAPOALIM_USER_CODE`, `SCRAPER_HAPOALIM_PASSWORD`
- **Visa Cal**: `SCRAPER_VISACAL_USERNAME`, `SCRAPER_VISACAL_PASSWORD`
- **Isracard**: `SCRAPER_ISRACARD_ID`, `SCRAPER_ISRACARD_CARD_6_DIGITS`, `SCRAPER_ISRACARD_PASSWORD`
- **Max**: `SCRAPER_MAX_USERNAME`, `SCRAPER_MAX_PASSWORD`
- **Beinleumi**: `SCRAPER_BEINLEUMI_USERNAME`, `SCRAPER_BEINLEUMI_PASSWORD`

See [israeli-bank-scrapers definitions](https://github.com/eshaham/israeli-bank-scrapers/blob/master/src/definitions.ts) for all companies and required fields.

## Usage

### 1. HTTP server (on-demand scrape)

Start the service (e.g. port 8010):

```bash
cd services/israeli-bank-scrapers-service && npm start
```

- **GET /api/health** — Liveness; returns `ledger_path` and `port`.
- **POST /scrape** — Run scraper and write to ledger. Optional body: `{ "companyId": "fibi" }`, `{ "companyId": "max" }`, `{ "companyId": "visaCal" }`, `{ "companyId": "discount", "startDate": "2024-01-01" }`. Credentials from env.

Example:

```bash
export SCRAPER_COMPANY_ID=discount
export SCRAPER_DISCOUNT_ID=your_id
export SCRAPER_DISCOUNT_PASSWORD=your_password
export SCRAPER_DISCOUNT_NUM=your_num
curl -X POST http://localhost:8010/scrape
```

Response: `{ "success": true, "written": 42, "accounts": 1, "ledger_path": "..." }`.

### 2. CLI (cron / manual)

Run once and write to ledger:

```bash
export SCRAPER_COMPANY_ID=discount
export SCRAPER_DISCOUNT_ID=...
export SCRAPER_DISCOUNT_PASSWORD=...
export SCRAPER_DISCOUNT_NUM=...
npm run scrape
```

Exit code 0 on success; stdout JSON `{ "success": true, "written", "accounts" }`.

### 3. After scrape: TUI / Web

The **Discount Bank service** (port 8003) reads bank accounts from the **ledger**. After a successful scrape, restart it or rely on the next poll; `GET http://localhost:8003/api/bank-accounts` will include the scraped accounts (e.g. `Assets:Bank:Discount:123456`).

## Ledger format

- Account path: `Assets:Bank:{BankName}:{accountNumber}` (e.g. `Assets:Bank:Discount:123456`).
- Each scraped transaction is stored as a double-entry ledger transaction (bank account + `Equity:Capital`).
- `metadata.source` = `israeli_bank_scrapers` for filtering.

## License

MIT. israeli-bank-scrapers is MIT.
