## 1Password Integration

The project can pull credentials directly from 1Password so secrets never land in source control. This includes:

- Distcc host credentials
- Cursor remote development credentials
- Alpaca API credentials
- Israeli bank scrapers credentials
- Other service credentials

### Two ways to read secrets

| Method | Use case | Where used in this project |
|--------|----------|----------------------------|
| **1Password CLI (`op`)** | Shell scripts, any language via subprocess | `scripts/include/onepassword.sh`, run-alpaca-service.sh, run_israeli_bank_scrapers_service.sh, op_sync_*.sh |
| **1Password SDKs (formal API)** | In-process code (Node, Python, Go) | Israeli bank scrapers service (optional), Python helper (optional) |

### Requirements (CLI – current default)

- [1Password CLI (`op`)](https://developer.1password.com/docs/cli)
- Authentication method (choose one):
  - **Personal account**: Signed-in session (`op signin …`) - for local development
  - **Service Account**: `OP_SERVICE_ACCOUNT_TOKEN` environment variable - for CI/CD and automation

- Optional: Cursor 1Password extension (for inline secret references)

See [1Password Secrets Automation](https://developer.1password.com/docs/secrets-automation) for details on Service Accounts vs Connect Servers.

### Formal API: 1Password SDKs

1Password provides **official SDKs** for programmatic access (no CLI subprocess):

- **Docs**: [1Password SDKs](https://developer.1password.com/docs/sdks/)
- **Load secrets**: [Load secrets using 1Password SDKs](https://developer.1password.com/docs/sdks/load-secrets)
- **Languages**: [Go](https://github.com/1Password/onepassword-sdk-go), [JavaScript](https://github.com/1Password/onepassword-sdk-js), [Python](https://github.com/1Password/onepassword-sdk-python)
- **Auth**: Service account token (`OP_SERVICE_ACCOUNT_TOKEN`) or [1Password desktop app](https://developer.1password.com/docs/sdks/concepts#1password-desktop-app) (biometric/local)

**In this project:**

- **Node (Israeli bank scrapers service)**: Uses [1Password JavaScript SDK](https://github.com/1Password/onepassword-sdk-js) (`@1password/sdk`) when `OP_SCRAPER_*_SECRET` env vars contain `op://` refs. Run the service or CLI directly (e.g. `npm start` or `npm run scrape`) with `OP_SERVICE_ACCOUNT_TOKEN` or desktop app; secrets are resolved in-process.
- **Python**: Optional helper `python/integration/onepassword_sdk_helper.py` uses [1Password Python SDK](https://github.com/1Password/onepassword-sdk-python) (`pip install onepassword-sdk`). Call `resolve_secret("op://vault/item/field")`, `resolve_secrets({"key": "op://..."})`, or `getenv_or_resolve(env_var, op_ref_env_var, default)` when you want in-process resolution instead of the shell wrapper + CLI. Auth: `OP_SERVICE_ACCOUNT_TOKEN` or `OP_1PASSWORD_ACCOUNT_NAME` (desktop app). The following Python clients resolve credentials optionally via 1Password when the corresponding `OP_*_SECRET` env vars are set to `op://` refs and the SDK is available (otherwise they use plain env or empty):
  - **Alpaca** (`alpaca_client.py`): `OP_ALPACA_CLIENT_ID_SECRET`, `OP_ALPACA_CLIENT_SECRET_SECRET`, `OP_ALPACA_API_KEY_ID_SECRET`, `OP_ALPACA_API_SECRET_KEY_SECRET`, `OP_ALPACA_ACCESS_TOKEN_SECRET`, `OP_ALPACA_REFRESH_TOKEN_SECRET`
  - **TradeStation** (`tradestation_client.py`): `OP_TRADESTATION_CLIENT_ID_SECRET`, `OP_TRADESTATION_CLIENT_SECRET_SECRET`, `OP_TRADESTATION_ACCOUNT_ID_SECRET`
  - **Tastytrade** (`tastytrade_client.py`): `OP_TASTYTRADE_CLIENT_SECRET_SECRET`, `OP_TASTYTRADE_REFRESH_TOKEN_SECRET`, `OP_TASTYTRADE_USERNAME_SECRET`, `OP_TASTYTRADE_PASSWORD_SECRET`
  - **SOFR/Treasury (FRED)** (`sofr_treasury_client.py`): `OP_FRED_API_KEY_SECRET`
  - **JupyterLab** (`jupyterlab_service.py`): `OP_JUPYTERLAB_TOKEN_SECRET`, `OP_JUPYTERLAB_PASSWORD_SECRET`
- **Shell scripts**: Continue to use the CLI via `scripts/include/onepassword.sh`; no change required.

**When to use SDK vs CLI**

- Use **CLI** when: running from shell scripts, or you already have `op signin` and want minimal setup.
- Use **SDK** when: building a long-running service (e.g. Node/Python) and you prefer not to spawn `op` or want a single dependency (SDK) without requiring the CLI to be installed.

## Israeli Bank Scrapers (israeli-bank-scrapers-service)

Use 1Password for scraper credentials (Discount, Leumi, Hapoalim, etc.) when running the Israeli bank scrapers service or CLI:

```bash
# Set 1Password secret references (op://Vault/Item/Field)
export OP_SCRAPER_DISCOUNT_ID_SECRET="op://Vault/Israeli Bank Discount/Identification Number"
export OP_SCRAPER_DISCOUNT_PASSWORD_SECRET="op://Vault/Israeli Bank Discount/password"
export OP_SCRAPER_DISCOUNT_NUM_SECRET="op://Vault/Israeli Bank Discount/code"

# Start the scrapers HTTP service (port 8010); credentials are loaded from 1Password
./scripts/run_israeli_bank_scrapers_service.sh

# Or run a one-off scrape and write to ledger
./scripts/run_israeli_bank_scrapers_service.sh scrape
```

When you start the service via `./scripts/service_manager.sh start israeli_bank_scrapers`, the same wrapper runs and will use 1Password if `OP_SCRAPER_*_SECRET` variables are set.

Supported refs: `OP_SCRAPER_DISCOUNT_ID_SECRET`, `OP_SCRAPER_DISCOUNT_PASSWORD_SECRET`, `OP_SCRAPER_DISCOUNT_NUM_SECRET`; Leumi: `OP_SCRAPER_LEUMI_USERNAME_SECRET`, `OP_SCRAPER_LEUMI_PASSWORD_SECRET`; Hapoalim: `OP_SCRAPER_HAPOALIM_USER_CODE_SECRET`, `OP_SCRAPER_HAPOALIM_PASSWORD_SECRET`; generic: `OP_SCRAPER_USERNAME_SECRET`, `OP_SCRAPER_PASSWORD_SECRET`, `OP_SCRAPER_ID_SECRET`, `OP_SCRAPER_NUM_SECRET`. See `services/israeli-bank-scrapers-service/README.md`.

### Sync distcc host from 1Password

Use `scripts/op_sync_distcc_host.sh` to populate:

- `ansible/hosts`
- `~/.ssh/<alias>_id_ed25519` and SSH config
- `~/.distcc/hosts`
- `~/.zshrc` (`DISTCC_HOSTS` export)

```bash
export OP_DISTCC_HOST_SECRET="op://Engineering/Distcc M4/host"
export OP_DISTCC_USER_SECRET="op://Engineering/Distcc M4/username"
export OP_DISTCC_KEY_SECRET="op://Engineering/Distcc M4/private key"

# optional

export OP_DISTCC_CORES_SECRET="op://Engineering/Distcc M4/cores"
export DISTCC_REMOTE_ALIAS="distcc-m4"

./scripts/op_sync_distcc_host.sh
```

Then run the provisioning playbook:

```bash
ansible-playbook -i ansible/hosts ansible/playbooks/setup_distcc_macos.yml
```

### Cursor references

You can reference the same secrets inside Cursor prompts using the extension, e.g.:

```
op://Engineering/Distcc M4/host
op://Engineering/Distcc M4/username
op://Engineering/Distcc M4/private key
```

### Notes

- `OP_DISTCC_*` variables accept any 1Password item paths.
- The script rewrites `ansible/hosts` for the `distcc_macos_workers` group each run.
- Update `DISTCC_REMOTE_ALIAS` or `DISTCC_REMOTE_CORES` to match new hosts.

### Sync Cursor remote development from 1Password

Use `scripts/op_sync_cursor_remote.sh` to populate SSH config for Cursor remote development:

- `~/.ssh/<alias>_id_ed25519` and SSH config
- Cursor-optimized SSH settings (compression, keep-alive, connection multiplexing)

```bash
export OP_CURSOR_REMOTE_HOST_SECRET="op://Engineering/Cursor Remote M4/host"
export OP_CURSOR_REMOTE_USER_SECRET="op://Engineering/Cursor Remote M4/username"
export OP_CURSOR_REMOTE_KEY_SECRET="op://Engineering/Cursor Remote M4/private key"

# optional

export OP_CURSOR_REMOTE_PORT_SECRET="op://Engineering/Cursor Remote M4/port"
export CURSOR_REMOTE_ALIAS="cursor-m4-mac"

./scripts/op_sync_cursor_remote.sh
```

After running the script:

1. Install Remote-SSH extension in Cursor (`anysphere.remote-ssh`) if not already installed
2. In Cursor, open Command Palette (`⌘+Shift+P`) and select "Remote-SSH: Connect to Host"
3. Choose your configured alias (e.g., `cursor-m4-mac`) from the list
4. Wait for VS Code Server to install on remote Mac (first connection only)

**SSH Settings Included:**

- Compression enabled for better performance over slow networks
- Keep-alive settings to prevent connection timeouts
- Connection multiplexing for faster subsequent connections
- Strict host key checking with auto-accept for first connection

See [Remote Development Workflow](./REMOTE_DEVELOPMENT_WORKFLOW.md) for complete setup instructions.

### Additional Notes

- `OP_CURSOR_REMOTE_*` variables accept any 1Password item paths.
- The script updates `~/.ssh/config` with Cursor-optimized settings.
- Update `CURSOR_REMOTE_ALIAS` to match your preferred SSH host alias.

## Alpaca API Credentials

Use 1Password for Alpaca API credentials when running the PWA service:

```bash
export OP_ALPACA_API_KEY_ID_SECRET="op://Vault/Item Name/API Key ID"
export OP_ALPACA_API_SECRET_KEY_SECRET="op://Vault/Item Name/API Secret Key"

./web/scripts/run-alpaca-service.sh
```

The script will automatically:

1. Try to read from 1Password if `OP_ALPACA_*_SECRET` variables are set
2. Fall back to `ALPACA_API_KEY_ID` and `ALPACA_API_SECRET_KEY` environment variables if 1Password is not available

**Authentication Methods:**

- **Personal Account**: Run `op signin` first (for local development)
- **Service Account**: Set `OP_SERVICE_ACCOUNT_TOKEN` (for CI/CD, see [Service Accounts docs](https://developer.1password.com/docs/service-accounts))

See `web/ALPACA_INTEGRATION.md` for complete setup instructions.
