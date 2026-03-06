# Backend Secrets: Providers and Generate/Configure Option

This project supports **generating and configuring backend secrets** (API keys, OAuth credentials, etc.) via a preferred provider. Use one provider; the tooling outputs the right env vars or config for your choice.

## If you have FRED or Alpaca API keys

Set them so the Risk-Free Rate service and Alpaca service can run:

| Keys | Env vars (plain) | 1Password (op://) |
|------|------------------|-------------------|
| **FRED** | `FRED_API_KEY=your_fred_key` | `OP_FRED_API_KEY_SECRET=op://Vault/FRED API/credential` |
| **Alpaca** | `ALPACA_API_KEY_ID=...` and `ALPACA_API_SECRET_KEY=...` | `OP_ALPACA_API_KEY_ID_SECRET=op://Vault/Alpaca/API Key ID` and `OP_ALPACA_API_SECRET_KEY_SECRET=op://Vault/Alpaca/API Secret Key` |

Then start the services (e.g. `./scripts/service.sh start riskfree`, `./scripts/service.sh start alpaca`) or run the TUI; the setup screen and status bar will show Health/Enabled when keys are present.

## Multiple keys per provider (paper / live / sandbox)

Some providers use **different keys or config** for paper vs live (or sandbox vs production). Use one set at a time per process; switch via env or config.

| Provider | Paper / sandbox | Live / production | How to switch |
|----------|-----------------|-------------------|---------------|
| **Alpaca** | Separate API keys from Alpaca **Paper** → API Keys | Separate keys from **Live** → API Keys | `ALPACA_PAPER=1` (default) uses paper keys + `paper-api.alpaca.markets`; `ALPACA_PAPER=0` uses live keys + `api.alpaca.markets`. Use two 1Password items (e.g. "Alpaca Paper", "Alpaca Live") and set the corresponding `OP_ALPACA_*_SECRET` for the mode you want. |
| **IBKR / TWS** | Gateway port **7497** (paper) | Gateway port **7496** (live) | One Gateway login; config or `tcp_backend_ports.tws` / gateway port selects 7497 vs 7496. No separate API keys; same TWS/Client Portal credentials. |
| **Tastytrade** | **Sandbox**: `TASTYTRADE_SANDBOX_BASE_URL`, cert.tastyworks.com | **Production**: `TASTYTRADE_BASE_URL`, api.tastytrade.com | Env or config `base_url` / sandbox flag. Credentials may differ (sandbox account vs live). Use separate 1Password items if you have both. |
| **TradeStation** | **SIM** (paper) base URL | **Live** base URL | `TRADESTATION_BASE_URL` or config; OAuth client may be per environment. |

Use one set of keys per run (e.g. only paper or only live) to avoid mixing environments.

## Recommended: 1Password

**Best option for this repo:** [1Password](https://1password.com/) with CLI and optional SDK. Already integrated for Alpaca, Tastytrade, TradeStation, FRED, Israeli bank scrapers, and more.

- **Generate**: Use 1Password app or CLI to create items; use `./scripts/setup_op_service_account.sh generate-and-configure` to create placeholder items and optionally generate strong passwords.
- **Configure**: Export `OP_*_SECRET` env vars pointing to `op://Vault/Item/field`; see [ONEPASSWORD_INTEGRATION.md](ONEPASSWORD_INTEGRATION.md).

**Quick start:**

```bash
./scripts/setup_op_service_account.sh setup-full          # First-time: token + placeholder items
./scripts/setup_op_service_account.sh generate-and-configure   # Generate secrets + print exports
```

---

## Alternative providers

Use these if your team standardizes on another secret manager. Configure your backend services with the provider’s env vars or config format; the **generate-and-configure** script is 1Password-oriented but the table below can guide manual setup.

| Provider | Best for | Generate | Configure in this project |
|----------|----------|----------|---------------------------|
| **[1Password](https://developer.1password.com/docs/cli/)** | Teams, CLI/SDK, op:// refs | CLI `op item create`, script `generate-and-configure` | `OP_*_SECRET=op://Vault/Item/field`, `config.json` values |
| **[HashiCorp Vault](https://www.vaultproject.io/)** | Self-hosted, Kubernetes, dynamic secrets | Vault KV or PKI engines | `VAULT_*` env or Vault Agent; map to `ALPACA_*`, `TASTYTRADE_*` etc. |
| **[Doppler](https://www.doppler.com/)** | App config + secrets, sync to env | Doppler dashboard or API | `DOPPLER_*` or Doppler inject; map to broker env vars |
| **[Infisical](https://infisical.com/)** | Open-source, self-hosted or cloud | Infisical dashboard or CLI | Export to env or mount; map to `ALPACA_*`, etc. |
| **[AWS Secrets Manager](https://aws.amazon.com/secrets-manager/)** | AWS workloads | AWS Console, CLI, or SDK | IAM + `aws secretsmanager get-secret-value`; inject into env |
| **[GCP Secret Manager](https://cloud.google.com/secret-manager)** | GCP workloads | gcloud or API | Service account + Secret Manager; inject into env |
| **[Azure Key Vault](https://azure.microsoft.com/products/key-vault)** | Azure workloads | Azure Portal or SDK | Managed identity or client secret; map to app config |
| **Env file + encryption** | Minimal deps, single machine | Manual or script | `.env` (gitignored), optionally encrypted (e.g. sops, age) |

---

## Generate and configure (1Password)

The script **generate-and-configure**:

1. **Generates** secure random values (e.g. for new API credentials or passwords) when creating items.
2. **Creates or updates** 1Password items for backends (Alpaca, Tastytrade, TradeStation, FRED, etc.).
3. **Outputs** the exact `OP_*_SECRET` exports and optional `config.json` snippets so you can copy into your shell or config.

### Usage

```bash
# Interactive: choose vault, which backends to create, and whether to generate passwords
./scripts/setup_op_service_account.sh generate-and-configure

# Non-interactive: use existing vault and create all default items
OP_SETUP_VAULT=MyVault ./scripts/setup_op_service_account.sh generate-and-configure --all

# Write exports to a file (e.g. for sourcing)
./scripts/setup_op_service_account.sh generate-and-configure --output-env .env.op
```

### What gets created

- **Placeholder items** (if missing): Alpaca API, Tastytrade, TradeStation API, FRED API, Alpha Vantage API, Finnhub API.
- **Generated values**: For new items you can opt to generate a strong password (or leave placeholder); broker API keys still come from the broker’s dashboard (Alpaca, IBKR, etc.).
- **Exports**: Script prints (and optionally writes) `export OP_ALPACA_API_KEY_ID_SECRET="op://Vault/Item/field"` etc.

### Broker API keys (Alpaca, IBKR, Tastytrade, TradeStation)

Real API keys are **issued by the broker**, not generated locally. Use:

1. **Generate-and-configure** to create 1Password items and get the `OP_*_SECRET` exports.
2. Log into the broker’s developer portal (Alpaca, IBKR, Tastytrade, TradeStation) and create API keys or OAuth apps.
3. Put the key/secret into the 1Password item (or paste when the script prompts).

---

## Security notes

- Never commit secrets or `OP_SERVICE_ACCOUNT_TOKEN` to the repo.
- Prefer `op://` references over plain env vars so secrets stay in 1Password.
- Use a dedicated vault and service account with minimal scope (e.g. read_items only) for automation.
- See [ONEPASSWORD_INTEGRATION.md](ONEPASSWORD_INTEGRATION.md) and [.cursorrules](/.cursorrules) for more.
