# Set Up Alpaca Secrets

Two ways to configure Alpaca credentials for the IB Box Spread services and TUI.

---

## Option A: 1Password (recommended)

### 1. Create the 1Password item and get exports

From the repo root:

```bash
# If you don't have a 1Password CLI token yet:
./scripts/setup_op_service_account.sh setup-full
# Then run:

./scripts/setup_op_service_account.sh generate-and-configure
```

When prompted, enter the vault name where you want the "Alpaca API" item. The script creates the item (if missing) and prints lines like:

```bash
export OP_ALPACA_API_KEY_ID_SECRET="op://YourVault/Alpaca API/username"
export OP_ALPACA_API_SECRET_KEY_SECRET="op://YourVault/Alpaca API/credential"
```

Copy those into your shell profile (`~/.zshrc` or `~/.bashrc`) or source them before running the Alpaca service.

Optional: write exports to a file and source when needed:

```bash
./scripts/setup_op_service_account.sh generate-and-configure --output-env .env.alpaca
source .env.alpaca
```

### 2. Get your Alpaca API keys

1. Go to [Alpaca Dashboard](https://app.alpaca.markets/) and sign in.
2. For **paper trading**: Paper → API Keys → Generate Key. Copy the **Key ID** and **Secret**.
3. For **live**: Live → API Keys (use with caution).

### 3. Put the keys into 1Password

1. Open 1Password and find the **Alpaca API** item (in the vault you used).
2. Set **username** to your Alpaca **API Key ID** (e.g. `PK...`).
3. Set **credential** to your Alpaca **API Secret Key**.

### 4. Run the Alpaca service

```bash
source ./scripts/setup_op_service_account.sh   # load OP token
The old `run-alpaca-service.sh` runtime path is retired.
```

Or with the env file:

```bash
source .env.alpaca
source ./scripts/setup_op_service_account.sh
The old `run-alpaca-service.sh` runtime path is retired.
```

Use **paper** by default: `ALPACA_PAPER=1` (default). For live: `export ALPACA_PAPER=0`.

---

## Option B: Environment variables (no 1Password)

You can use either **API keys** (from the dashboard) or **OAuth** (client_id + client_secret from an OAuth app). If both are set, OAuth is used. See [ALPACA_OAUTH.md](ALPACA_OAUTH.md) for OAuth setup.

### 1. Get your Alpaca API keys

Same as above: [Alpaca Dashboard](https://app.alpaca.markets/) → Paper (or Live) → API Keys → Generate and copy Key ID and Secret.

### 2. Export in your shell

```bash
export ALPACA_API_KEY_ID="PKxxxxxxxxxxxxxxxxxx"
export ALPACA_API_SECRET_KEY="your_secret_key_here"
export ALPACA_PAPER=1   # 1 = paper (default), 0 = live
```

Add these to `~/.zshrc` or `~/.bashrc` if you want them in every session (never commit this file with real keys).

### 3. Run the Alpaca service

```bash
The old `run-alpaca-service.sh` runtime path is retired.
```

---

## Verify

- **Health:** `curl -s http://127.0.0.1:8000/api/health` should show `"status": "ok"` and `"alpaca_connected": true` once the service is running and credentials are valid.
- **TUI:** Start the TUI with the same env (or 1Password exports); Alpaca should appear as configured in the status bar.

See also: [ONEPASSWORD_INTEGRATION.md](ONEPASSWORD_INTEGRATION.md),
[BACKEND_SECRETS_PROVIDERS.md](BACKEND_SECRETS_PROVIDERS.md),
[ALPACA_OAUTH.md](ALPACA_OAUTH.md),
[ALPACA_TASTYTRADE_RUNTIME_RETIREMENT.md](/Users/davidl/Projects/Trading/ib_box_spread_full_universal/docs/archive/ALPACA_TASTYTRADE_RUNTIME_RETIREMENT.md).
