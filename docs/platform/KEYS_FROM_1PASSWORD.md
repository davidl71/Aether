# Keys from 1Password (Rust backend & TUI)

How to load API keys and secrets from 1Password so the **Rust backend** and **TUI** get live data (e.g. FRED for benchmark yield curve). See historical docs for full integration: [ONEPASSWORD_INTEGRATION.md](../ONEPASSWORD_INTEGRATION.md), [BACKEND_SECRETS_PROVIDERS.md](../BACKEND_SECRETS_PROVIDERS.md).

## One-time setup

1. **1Password CLI**  
   Install [op](https://developer.1password.com/docs/cli) and sign in or use a service account.

2. **Service account (recommended, no repeated prompts)**  
   ```bash
   ./scripts/setup_op_service_account.sh setup-token    # create token, save to ~/.config/op/service_account_token
   source ./scripts/setup_op_service_account.sh        # load token + optional OP_FRED_API_KEY_SECRET
   ```

3. **Create secret items and get op:// refs**  
   ```bash
   ./scripts/setup_op_service_account.sh generate-and-configure   # creates FRED API, Alpaca, etc.; prints OP_*_SECRET exports
   ```  
   Copy the suggested `export OP_FRED_API_KEY_SECRET="op://Vault/FRED API/credential"` (and any others) into your shell profile or a file you source before running services.

## Env vars the Rust stack uses

| Key / purpose | Env var | 1Password ref (set this, then resolve before run) |
|---------------|---------|--------------------------------------------------|
| **FRED (benchmark yield curve)** | `FRED_API_KEY` | `OP_FRED_API_KEY_SECRET=op://Vault/FRED API/credential` |
| **FMP fundamentals** | `FMP_API_KEY` | (optional) Create item "FMP API", then `OP_FMP_API_KEY_SECRET=op://Vault/FMP API/credential` |
| **NATS / backend** | `NATS_URL`, `BACKEND_ID` | Not secret; use config or env. |

The Rust backend and TUI **do not** resolve `op://` refs themselves. You must **export the plain env var** (e.g. `FRED_API_KEY`) in the same shell where you start the backend or TUI.

## Auto-resolve in run scripts (implemented)

**`./scripts/run_rust_tui.sh`** and **`./scripts/service_manager.sh start rust_backend`** now resolve 1Password refs automatically: they source `scripts/include/onepassword.sh` and call `export_op_secrets_for_rust`, which sets `FRED_API_KEY` (and `FMP_API_KEY` if configured) from `OP_FRED_API_KEY_SECRET` / `OP_FMP_API_KEY_SECRET` when the `op` CLI is available and the refs are set. No manual export needed when using these scripts.

Ensure `OP_SERVICE_ACCOUNT_TOKEN` is set (or in `~/.config/op/service_account_token`) and `OP_FRED_API_KEY_SECRET` is set to your `op://Vault/FRED API/credential` ref—e.g. by running `source ./scripts/setup_op_service_account.sh` in the same shell before `./scripts/run_rust_tui.sh`, or by exporting them in your profile.

---

## Resolving and running (manual or other entrypoints)

**Option A – resolve in current shell, then start services**

```bash
# Load 1Password token (and optional OP_*_SECRET refs)
source ./scripts/setup_op_service_account.sh

# Resolve FRED key from 1Password and export for this shell
if [[ -n "${OP_FRED_API_KEY_SECRET:-}" ]]; then
  export FRED_API_KEY="$(op read "${OP_FRED_API_KEY_SECRET}" 2>/dev/null || true)"
fi

# Start backend and TUI in this shell (they will see FRED_API_KEY)
./scripts/start_all_services.sh
cd agents/backend && cargo run -p tui_service
```

**Option B – use `read_credential` from shared include**

```bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/include/onepassword.sh"

export FRED_API_KEY="$(read_credential "${OP_FRED_API_KEY_SECRET:-}" "${FRED_API_KEY:-}" 2>/dev/null || true)"
# Then start backend / TUI
```

**Option C – plain key (no 1Password)**  

If you don’t use 1Password for FRED:

```bash
export FRED_API_KEY="your_fred_api_key_here"
```

Then start the Rust backend and TUI in the same shell.

## CLI benchmarks (test FRED without backend/TUI)

To test FRED (SOFR + Treasury) from the command line without running the backend or TUI, run from **`agents/backend`** (the Rust workspace root):

```bash
# From repo root: cd into the workspace first
source ./scripts/include/onepassword.sh && export_op_secrets_for_rust
cd agents/backend
cargo run -p cli -- benchmarks          # pretty-printed summary
cargo run -p cli -- benchmarks --json  # full JSON
```

Or with a plain key:

```bash
export FRED_API_KEY="your_fred_api_key_here"
cd agents/backend && cargo run -p cli -- benchmarks
```

## References (historical docs)

- **1Password integration:** [docs/ONEPASSWORD_INTEGRATION.md](../ONEPASSWORD_INTEGRATION.md) – CLI vs SDKs, service account, Alpaca/Tastytrade/FRED/Israeli bank scrapers, `OP_*_SECRET` table.
- **Backend secrets providers:** [docs/BACKEND_SECRETS_PROVIDERS.md](../BACKEND_SECRETS_PROVIDERS.md) – FRED and Alpaca env vars, 1Password quick start, `generate-and-configure`.
- **NATS API (finance_rates):** [NATS_API.md](NATS_API.md) – `FRED_API_KEY` for live SOFR/Treasury via `api.finance_rates.benchmarks`, `.sofr`, `.treasury`.
- **TUI Yield tab:** [TUI_LEGACY_DESIGN_LEARNINGS.md](TUI_LEGACY_DESIGN_LEARNINGS.md) §5b – benchmark yield curve from `api.finance_rates.benchmarks` (FRED when key set).
