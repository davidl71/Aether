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
| **1Password CLI (`op`)** | Shell scripts, any language via subprocess | `scripts/include/onepassword.sh`, `run-ib-service.sh`, op_sync_*.sh |
| **1Password SDKs (formal API)** | In-process code (Node, Python, Go) | Israeli bank scrapers service (optional), Python helper (optional) |

### Why does 1Password keep asking me to authorize?

Repeated authorization usually means you're **not** using a long-lived token, so each new process or shell triggers auth again:

1. **Personal account (`op signin`)**  
   The CLI stores a **session** in your env (e.g. `OP_SESSION_...`). That session:
   - Is only in the shell where you ran `op signin` (or where you ran `eval $(op signin)`).
   - Expires (often after sleep, restart, or after a period of inactivity).
   - Is **not** inherited by new terminals, Cursor’s run commands, or services you start from the IDE.

2. **Desktop app / biometric**  
   If the Python SDK uses `OP_1PASSWORD_ACCOUNT_NAME` (desktop app), it may prompt for Touch ID / password per process or when the app’s cache isn’t used.

**Fix: use a Service Account token** so nothing prompts after the first setup:

- Create a service account and get a token once:  
  `./scripts/setup_op_service_account.sh setup-token`  
  (or [create it manually](https://developer.1password.com/docs/service-accounts/get-started#create-a-service-account)).
- Export the token in **every** environment where you run scripts or the TUI:

  ```bash
  export OP_SERVICE_ACCOUNT_TOKEN="ops_xxxxxxxx..."
  ```

  Or add that line to `~/.zshrc` / `~/.bashrc`, or use the script’s token file and source it:

  ```bash
  source ./scripts/setup_op_service_account.sh
  ```

- Then the CLI and the Python SDK use that token and **do not** ask for authorization again until the token is revoked or expires.

**Optional:** Persist the token in a file and load it when needed:

```bash
# One-time: save token to a file (chmod 600 recommended)
echo 'export OP_SERVICE_ACCOUNT_TOKEN="ops_..."' > ~/.config/op/service_account_token
# In each new shell or in ~/.zshrc:
source ~/.config/op/service_account_token
```

The setup script can save the token to `~/.config/op/service_account_token` (or `$OP_SERVICE_ACCOUNT_TOKEN_FILE`); run `source ./scripts/setup_op_service_account.sh` to load it into the current shell.

### Requirements (CLI – current default)

- [1Password CLI (`op`)](https://developer.1password.com/docs/cli)
- Authentication method (choose one):
  - **Personal account**: Signed-in session (`op signin …`) - for local development
  - **Service Account**: `OP_SERVICE_ACCOUNT_TOKEN` environment variable - for CI/CD and automation

- Optional: [1Password Cursor plugin](https://github.com/1Password/cursor-plugin) (Cursor Settings > Plugins: validate [1Password Environments](https://developer.1password.com/docs/environments) local `.env` files before shell execution) or Cursor 1Password extension (for inline secret references in prompts)

See [1Password Secrets Automation](https://developer.1password.com/docs/secrets-automation) for details on Service Accounts vs Connect Servers.

**Generate and configure backend secrets:** Use `./scripts/setup_op_service_account.sh generate-and-configure` to create 1Password items and print `OP_*_SECRET` exports. See [Backend secrets providers](BACKEND_SECRETS_PROVIDERS.md) for provider options (1Password recommended) and alternatives (HashiCorp Vault, Doppler, etc.).

**GitHub:** [1Password organization](https://github.com/orgs/1Password/repositories?type=all) — CLI source, [**onepassword-sdk-python**](https://github.com/1Password/onepassword-sdk-python) (Python SDK; [example](https://github.com/1Password/onepassword-sdk-python/tree/main/example)), [**onepassword-sdk-go**](https://github.com/1Password/onepassword-sdk-go) (Go SDK), [**onepassword-sdk-js**](https://github.com/1Password/onepassword-sdk-js) (JavaScript SDK), [shell-plugins](https://github.com/1Password/shell-plugins), [onepassword-operator](https://github.com/1Password/onepassword-operator) (Kubernetes/Connect), [**connect**](https://github.com/1Password/connect) (1Password Connect Server: REST API and CLI for apps/cloud; [get started](https://developer.1password.com/docs/connect/get-started)), [**typeshare**](https://github.com/1Password/typeshare) (Rust → Swift, Go, Kotlin, Scala, TypeScript, Python type generation for FFI; [book](https://1password.github.io/typeshare)), [**credential-exchange**](https://github.com/1Password/credential-exchange) (Rust libs for [FIDO Credential Exchange](https://fidoalliance.org/specifications-credential-exchange-specifications/) — passkey portability), [**crash-handling**](https://github.com/1Password/crash-handling) (Rust crates for crash context and minidumps), [**secrets-orb**](https://github.com/1Password/secrets-orb) ([CircleCI orb](https://circleci.com/orbs/registry/orb/onepassword/secrets): load secrets via Connect or Service Account), [**sys-locale**](https://github.com/1Password/sys-locale) (Rust crate: get system/application locale), and [**cursor-plugin**](https://github.com/1Password/cursor-plugin) (official [1Password Cursor plugin](https://cursor.com/marketplace): validate [1Password Environments](https://developer.1password.com/docs/environments) local `.env` mounts before shell execution), and [**cursor-hooks**](https://github.com/1Password/cursor-hooks) (Cursor hooks repo: [1password-validate-mounted-env-files](https://github.com/1Password/cursor-hooks/blob/main/.cursor/hooks/1password/README.md); plugin above is based on this). **Examples:** [solutions](https://github.com/1Password/solutions) (1Password Solutions team: SDK demos, migration, provisioning, user/vault/item management scripts).

**CLI reference:** [Management commands](https://developer.1password.com/docs/cli/reference/management-commands) (`op` subcommands: account, connect, item, service-account, vault, etc.). For the setup script, the relevant pages are [service-account](https://developer.1password.com/docs/cli/reference/management-commands/service-account) (e.g. `op service-account create`) and [item](https://developer.1password.com/docs/cli/reference/management-commands/item) (e.g. `op item create`, `op item get`). See [CLI best practices](https://developer.1password.com/docs/cli/best-practices) for secure usage.

### Create and connect a service account using the `op` CLI

You can create a service account and use it with the `op` CLI so scripts and this project can read secrets without a personal sign-in.

**Requirements:** 1Password CLI 2.18.0 or later (`op --version`). Service accounts require a 1Password membership (Teams/Business) with permission to create service accounts.

**1. Create a service account**

Use the CLI (after signing in with a personal account that has permission to create service accounts):

```bash
# Sign in first (one-time or when session expires)
op signin

# Create a service account with access to specific vaults.
# Grant read access to a vault (use vault name or ID):
op service-account create "My Automation" --vault "Engineering=read_items"

# Optional: allow write access in a vault
op service-account create "My Automation" --vault "Engineering=read_items,write_items"

# Optional: set an expiry (e.g. 90 days)
op service-account create "My Automation" --vault "Engineering=read_items" --expires-in 90d
```

The command outputs a **service account token** once. Save it in 1Password (e.g. create a Secure Note) or in your secret manager; you cannot retrieve it again.

**2. Connect the CLI to the service account**

Export the token so the `op` CLI uses it instead of your personal session:

```bash
# bash, zsh, sh
export OP_SERVICE_ACCOUNT_TOKEN="ops_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"

# Verify it works (list vaults or read a secret)
op vault list
op read "op://Engineering/Some Item/credential"
```

**3. Use with this project**

Once `OP_SERVICE_ACCOUNT_TOKEN` is set, any script that uses the CLI via `scripts/include/onepassword.sh` (e.g. `read_credential`) or that runs `op read` will use the service account. For example:

```bash
export OP_SERVICE_ACCOUNT_TOKEN="ops_..."
export OP_ALPACA_API_KEY_ID_SECRET="op://Vault/Alpaca/API Key ID"
export OP_ALPACA_API_SECRET_KEY_SECRET="op://Vault/Alpaca/API Secret Key"

Alpaca runtime launchers are retired.
```

The Python SDK (`onepassword_sdk_helper.py`) also uses `OP_SERVICE_ACCOUNT_TOKEN` when set, so the TUI and backend services can resolve `op://` refs without the CLI.

**Notes**

- If you use 1Password Connect, `OP_CONNECT_HOST` and `OP_CONNECT_TOKEN` take precedence over `OP_SERVICE_ACCOUNT_TOKEN`; unset them to use the service account.
- Vault access and permissions are set at creation time and cannot be changed; create a new service account to change access.
- **Setup script:** Run `source ./scripts/setup_op_service_account.sh` to load the token from `OP_SERVICE_ACCOUNT_TOKEN` or from `~/.config/op/service_account_token` (or `$OP_SERVICE_ACCOUNT_TOKEN_FILE`) and export it into your shell. Use `./scripts/setup_op_service_account.sh verify` to test the connection. Optionally run the `op` CLI to create the token and secret items:
  - **`./scripts/setup_op_service_account.sh setup-token`** — Sign in with a personal account, create a service account (`op service-account create`), and save the token to the token file. Set `OP_SETUP_VAULT` or enter the vault name when prompted.
  - **`./scripts/setup_op_service_account.sh setup-secrets`** — Sign in, then create placeholder 1Password items (Alpaca API, Tastytrade, FRED API) in a vault and print suggested `OP_*_SECRET` exports. Set `OP_SETUP_VAULT` or enter the vault name when prompted.
  - **`./scripts/setup_op_service_account.sh setup-full`** — Interactive only: runs setup-token, then setup-secrets, then export. Use for first-time setup.
- **Config missing:** The old Python TUI launcher used to trigger `./scripts/setup_op_service_account.sh setup-full` when the shared config file was missing. The active Rust TUI launcher does not do that bootstrap flow automatically; run the setup script yourself first, then start `./scripts/run_rust_tui.sh` with either a real shared config file or explicit env overrides.
- Official docs: [Create a service account](https://developer.1password.com/docs/service-accounts/get-started#create-a-service-account), [Use service accounts with 1Password CLI](https://developer.1password.com/docs/service-accounts/use-with-1password-cli).

### Formal API: 1Password SDKs

1Password provides **official SDKs** for programmatic access (no CLI subprocess):

- **Docs**: [1Password SDKs](https://developer.1password.com/docs/sdks/)
- **Load secrets**: [Load secrets using 1Password SDKs](https://developer.1password.com/docs/sdks/load-secrets)
- **Languages**: [Go](https://github.com/1Password/onepassword-sdk-go), [JavaScript](https://github.com/1Password/onepassword-sdk-js), [Python](https://github.com/1Password/onepassword-sdk-python)
- **Python sample app**: [onepassword-sdk-python/example](https://github.com/1Password/onepassword-sdk-python/tree/main/example) — run `python example/example.py` after cloning; demonstrates auth with service account token, list vaults/items, resolve secrets and TOTP, create/update items, password generation, sharing, SSH keys, documents. Prerequisites: `export OP_SERVICE_ACCOUNT_TOKEN="<token>"` and `export OP_VAULT_ID="<vault uuid>"`.
- **Auth**: Service account token (`OP_SERVICE_ACCOUNT_TOKEN`) or [1Password desktop app](https://developer.1password.com/docs/sdks/concepts#1password-desktop-app) (biometric/local)

**Patterns from the official Python example (vs this project)**

| Pattern | Official example | This project | Takeaway |
|--------|-------------------|--------------|----------|
| **Client lifecycle** | Create client once in `main()`, pass into helpers | Global cached client in `onepassword_sdk_helper` | Our cache is better for long-lived TUI/services; example is better for short scripts. |
| **Validate ref before use** | `Secrets.validate_secret_reference("op://vault/item/field")` before resolve | No validation; we pass ref straight to resolve | Add optional validation to fail fast with a clear error on bad syntax. |
| **Bulk resolve** | `client.secrets.resolve_all([ref1, ref2])` — one round-trip | `resolve_secrets()` loops over `resolve_secret()` — N round-trips | Use SDK `resolve_all` when resolving multiple refs and SDK is available (fewer network calls). |
| **TOTP** | `client.secrets.resolve("op://.../field?attribute=totp")` for one-time codes | Not implemented | Add only if you need TOTP (e.g. 2FA in automation). |
| **List vaults/items** | `client.vaults.list()`, `client.items.list(vault_id)` | TUI uses subprocess `op vault list` when SDK auth fails | When SDK works, could use SDK list for 1Password tab (no CLI dependency). |
| **CLI fallback** | Example assumes SDK only | We fall back to `op read` when SDK fails | Our fallback improves robustness when SDK is missing or auth misconfigured. |
| **Env + op://** | Example uses raw token + vault ID | `getenv_or_resolve(env_var, op_ref_env_var, default)` | Our helper fits config loading and client constructors (env first, then op://). |
| **Batch item ops** | `client.items.create_all`, `get_all`, `delete_all` | We only read secrets | Adopt batch APIs if we ever create/update many items from Python. |

**Implementations to consider:** In `onepassword_sdk_helper.py`: (1) optionally call `Secrets.validate_secret_reference` before resolve for better errors; (2) implement `resolve_secrets()` via `client.secrets.resolve_all()` when the SDK is available so multiple refs use one round-trip.

**Done in this project:** (1) `validate_secret_reference(ref)` is implemented and can be used with `resolve_secret(ref, validate=True)`. (2) `resolve_secrets()` uses `client.secrets.resolve_all()` when the SDK is available, then fills any missing keys via single resolves.

**In this project:**

| Backend | 1Password env var(s) | op:// path (Vault/Item/field) |
|---------|----------------------|--------------------------------|
| **Alpaca** | `OP_ALPACA_API_KEY_ID_SECRET`, `OP_ALPACA_API_SECRET_KEY_SECRET` | Vault / Alpaca (or "Alpaca Paper", "Alpaca Live") / API Key ID, API Secret Key |
| | `OP_ALPACA_CLIENT_ID_SECRET`, `OP_ALPACA_CLIENT_SECRET_SECRET` (OAuth) | same item or OAuth item / client_id, client_secret |
| | `OP_ALPACA_ACCESS_TOKEN_SECRET`, `OP_ALPACA_REFRESH_TOKEN_SECRET` | optional / token fields |
| **Tastytrade** | `OP_TASTYTRADE_USERNAME_SECRET`, `OP_TASTYTRADE_PASSWORD_SECRET` | Vault / Tastytrade / username, password |
| | `OP_TASTYTRADE_CLIENT_SECRET_SECRET`, `OP_TASTYTRADE_REFRESH_TOKEN_SECRET` (OAuth) | same or OAuth item / client_secret, refresh_token |
| **FRED (SOFR/Treasury)** | `OP_FRED_API_KEY_SECRET` | Vault / FRED API / credential or api_key |
| **Alpha Vantage** | `OP_ALPHA_VANTAGE_API_KEY_SECRET` | Vault / Alpha Vantage API / credential |
| **Finnhub** | `OP_FINNHUB_API_KEY_SECRET` | Vault / Finnhub API / credential |

- **Node (Israeli bank scrapers service)**: Uses [1Password JavaScript SDK](https://github.com/1Password/onepassword-sdk-js) when `OP_SCRAPER_*_SECRET` env vars contain `op://` refs. See table below.
- **Python**: Optional helper `python/integration/onepassword_sdk_helper.py` uses [1Password Python SDK](https://github.com/1Password/onepassword-sdk-python). Call `resolve_secret("op://vault/item/field")`, `resolve_secrets({"key": "op://..."})`, or `getenv_or_resolve(env_var, op_ref_env_var, default)`. Auth: `OP_SERVICE_ACCOUNT_TOKEN` or `OP_1PASSWORD_ACCOUNT_NAME` (desktop app).
- **Shell scripts**: Use the CLI via `scripts/include/onepassword.sh`.

**When to use SDK vs CLI**

- Use **CLI** when: running from shell scripts, or you already have `op signin` and want minimal setup.
- Use **SDK** when: building a long-running service (e.g. Node/Python) and you prefer not to spawn `op` or want a single dependency (SDK) without requiring the CLI to be installed.

**Scripts vs in-process resolution:** Service start scripts resolve refs via the CLI and export env vars; the process then uses those env vars and does not resolve again. So use either the script (CLI) or run the process directly with `OP_*_SECRET` set (SDK), not both in the same flow.

### TUI and PWA (web app)

- **TUI (Rust)**: The active Rust TUI uses the same shared config and env surface as the backend services. Resolve or export any needed secrets first, then start `./scripts/run_rust_tui.sh`. If your shared config contains `op://` references, resolve them before launch through the setup flow or another process that materializes the needed env/config values.
- **Shared config file**: The shared config loader (`SharedConfigLoader`) resolves **op://** references in config values when the 1Password Python SDK is installed. You can put `op://Vault/Item/field` in `config/config.json` (or the file pointed to by `IB_BOX_SPREAD_CONFIG`) for any secret field (e.g. under `alpaca.data_client_config.api_key_id`, `tastytrade.oauth.client_secret`). At load time those values are resolved in-process; if the SDK is missing or auth fails, the value is left as-is (and may cause validation errors).
- **PWA / web app**: The web runtime is retired. If web returns later, credentials should still stay in backend processes rather than the frontend build.

## Israeli Bank Scrapers (israeli-bank-scrapers-service)

Use 1Password for scraper credentials when running the Israeli bank scrapers service or CLI. Set the env vars below to `op://Vault/Item/field`, then start the service or run a one-off scrape.

| Company / service | 1Password env var(s) | 1Password item field(s) |
|------------------|----------------------|--------------------------|
| **Discount** | `OP_SCRAPER_DISCOUNT_ID_SECRET`, `OP_SCRAPER_DISCOUNT_PASSWORD_SECRET`, `OP_SCRAPER_DISCOUNT_NUM_SECRET` | Identification Number, password, code |
| **Leumi** | `OP_SCRAPER_LEUMI_USERNAME_SECRET`, `OP_SCRAPER_LEUMI_PASSWORD_SECRET` | username, password |
| **Hapoalim** | `OP_SCRAPER_HAPOALIM_USER_CODE_SECRET`, `OP_SCRAPER_HAPOALIM_PASSWORD_SECRET` | user code, password |
| **Visa Cal** | `OP_SCRAPER_VISACAL_USERNAME_SECRET`, `OP_SCRAPER_VISACAL_PASSWORD_SECRET` | username, password |
| **Isracard** | `OP_SCRAPER_ISRACARD_ID_SECRET`, `OP_SCRAPER_ISRACARD_CARD_6_DIGITS_SECRET`, `OP_SCRAPER_ISRACARD_PASSWORD_SECRET` | id, card 6 digits, password |
| **Max** | `OP_SCRAPER_MAX_USERNAME_SECRET`, `OP_SCRAPER_MAX_PASSWORD_SECRET` | username, password |
| **Beinleumi** | `OP_SCRAPER_BEINLEUMI_USERNAME_SECRET`, `OP_SCRAPER_BEINLEUMI_PASSWORD_SECRET` | username, password |
| **Generic fallback** | `OP_SCRAPER_USERNAME_SECRET`, `OP_SCRAPER_PASSWORD_SECRET`, `OP_SCRAPER_ID_SECRET`, `OP_SCRAPER_NUM_SECRET`, `OP_SCRAPER_CARD_6_DIGITS_SECRET` | username, password, id, code, card 6 digits |

**Example (Discount):**

```bash
export OP_SCRAPER_DISCOUNT_ID_SECRET="op://Vault/Israeli Bank Discount/Identification Number"
export OP_SCRAPER_DISCOUNT_PASSWORD_SECRET="op://Vault/Israeli Bank Discount/password"
export OP_SCRAPER_DISCOUNT_NUM_SECRET="op://Vault/Israeli Bank Discount/code"

./scripts/run_israeli_bank_scrapers_service.sh
# Or one-off: ./scripts/run_israeli_bank_scrapers_service.sh scrape
```

When you start the service via `./scripts/service_manager.sh start israeli_bank_scrapers`, the same wrapper runs and will use 1Password if `OP_SCRAPER_*_SECRET` variables are set. See `services/israeli-bank-scrapers-service/README.md`.

### Sync distcc host from 1Password

Use `scripts/op_sync_distcc_host.sh` to populate ansible hosts, SSH keys, and distcc config. Set these env vars to `op://Vault/Item/field`:

| Env var | Purpose |
|---------|---------|
| `OP_DISTCC_HOST_SECRET` | Remote host (hostname or IP) |
| `OP_DISTCC_USER_SECRET` | SSH username |
| `OP_DISTCC_KEY_SECRET` | SSH private key |
| `OP_DISTCC_CORES_SECRET` | (Optional) Cores to use |
| `DISTCC_REMOTE_ALIAS` | (Optional) SSH config alias |

**Example:**

```bash
export OP_DISTCC_HOST_SECRET="op://Engineering/Distcc M4/host"
export OP_DISTCC_USER_SECRET="op://Engineering/Distcc M4/username"
export OP_DISTCC_KEY_SECRET="op://Engineering/Distcc M4/private key"
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

Use `scripts/op_sync_cursor_remote.sh` to populate SSH config for Cursor remote development. Set these env vars to `op://Vault/Item/field`:

| Env var | Purpose |
|---------|---------|
| `OP_CURSOR_REMOTE_HOST_SECRET` | Remote host (hostname or IP) |
| `OP_CURSOR_REMOTE_USER_SECRET` | SSH username |
| `OP_CURSOR_REMOTE_KEY_SECRET` | SSH private key |
| `OP_CURSOR_REMOTE_PORT_SECRET` | (Optional) SSH port |
| `CURSOR_REMOTE_ALIAS` | (Optional) SSH config alias |

**Example:**

```bash
export OP_CURSOR_REMOTE_HOST_SECRET="op://Engineering/Cursor Remote M4/host"
export OP_CURSOR_REMOTE_USER_SECRET="op://Engineering/Cursor Remote M4/username"
export OP_CURSOR_REMOTE_KEY_SECRET="op://Engineering/Cursor Remote M4/private key"
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

Use 1Password for Alpaca API credentials when running the PWA or backend services. Map env vars to `op://Vault/Item/field`:

| Env var | Purpose | 1Password field (typical) |
|---------|---------|----------------------------|
| `OP_ALPACA_API_KEY_ID_SECRET` | Alpaca API Key ID | API Key ID |
| `OP_ALPACA_API_SECRET_KEY_SECRET` | Alpaca API Secret Key | API Secret Key |

For **paper vs live**: use separate 1Password items (e.g. "Alpaca Paper", "Alpaca Live") and set the corresponding `OP_ALPACA_*_SECRET` for the mode you want; use `ALPACA_PAPER=1` or `0` to select endpoint. See [BACKEND_SECRETS_PROVIDERS.md](BACKEND_SECRETS_PROVIDERS.md#multiple-keys-per-provider-paper--live--sandbox).

**Example:**

```bash
export OP_ALPACA_API_KEY_ID_SECRET="op://Vault/Item Name/API Key ID"
export OP_ALPACA_API_SECRET_KEY_SECRET="op://Vault/Item Name/API Secret Key"

Alpaca runtime launchers are retired.
```

The script will: (1) read from 1Password if `OP_ALPACA_*_SECRET` are set, (2) fall back to `ALPACA_API_KEY_ID` and `ALPACA_API_SECRET_KEY` if 1Password is not available.

**Authentication:** Personal account — run `op signin` first; Service account — set `OP_SERVICE_ACCOUNT_TOKEN`. See [Service Accounts](https://developer.1password.com/docs/service-accounts).
