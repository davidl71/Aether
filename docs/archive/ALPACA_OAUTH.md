# Alpaca OAuth

This project supports **Alpaca OAuth** (client credentials) as an alternative to API keys. Use OAuth when you want to authenticate with **client_id** and **client_secret** (e.g. from an OAuth app you registered) instead of per-account API Key ID + Secret.

---

## Two OAuth contexts (Alpaca)

| Context | Purpose | What you get | Used in this project? |
|--------|---------|---------------|------------------------|
| **Connect API (end-user OAuth)** | Let *end users* connect their Alpaca accounts to your app | User authorizes in browser; you get tokens to act on their behalf | No — we use one app identity. |
| **Client credentials (app-only)** | Authenticate *your app* to Alpaca (no end-user login) | You register an OAuth app → get `client_id` + `client_secret` → exchange for access tokens | **Yes** — see below. |

Our Alpaca client uses the **client credentials** grant: `client_id` + `client_secret` → `POST /oauth/token` → Bearer token for Trading/Market Data API calls.

---

## How to use OAuth in this project

### 1. Register an OAuth app in Alpaca

1. Sign in at [app.alpaca.markets](https://app.alpaca.markets/).
2. Open **Develop** (or **API**) → **OAuth Apps** (or **Connect** / **Register Your App**).
3. Create a new OAuth app. Note the **Client ID** and **Client Secret** (Alpaca may show the secret only once — store it securely).
4. For **paper** trading, use the app in the paper environment; for **live**, ensure the app is approved for live if required.

Alpaca’s docs: [About Connect API](https://docs.alpaca.markets/docs/about-connect-api), [OAuth Integration Guide](https://alpaca.markets/docs/build-apps_services-with-alpaca/oauth-guide/) (path may vary).

### 2. Configure with environment variables

```bash
# OAuth (client credentials) – preferred when you have an OAuth app
export ALPACA_CLIENT_ID="your_oauth_client_id"
export ALPACA_CLIENT_SECRET="your_oauth_client_secret"
export ALPACA_PAPER=1   # 1 = paper (default), 0 = live
```

If both OAuth and API keys are set, the client uses **OAuth** first.

### 3. Configure with 1Password

Store the OAuth app credentials in 1Password and reference them:

```bash
export OP_ALPACA_CLIENT_ID_SECRET="op://Vault/Alpaca OAuth App/Client ID"
export OP_ALPACA_CLIENT_SECRET_SECRET="op://Vault/Alpaca OAuth App/Client Secret"
source ./scripts/setup_op_service_account.sh
```

Use the same vault/item in `generate-and-configure` if you create an “Alpaca OAuth” item (e.g. fields `Client ID`, `Client Secret`).

### 4. Runtime status

The old standalone Alpaca Python service is retired. OAuth details remain useful for the retained
Alpaca client/helper code, but there is no supported `run-alpaca-service.sh` runtime path now.

---

## Implementation details

- **Client:** `python/integration/alpaca_client.py`
  - If `client_id` and `client_secret` are set → `_use_oauth = True`, no API key headers.
  - Token endpoint: `https://paper-api.alpaca.markets/oauth/token` (paper) or `https://api.alpaca.markets/oauth/token` (live).
  - Grant type: `client_credentials`.
  - Access token is cached and refreshed when near expiry (or on 401).
- **Retained helper:** `python/integration/alpaca_client.py`
  still supports OAuth-capable client logic for future reuse.
- **Official SDK:** `alpaca-py` is used only when **not** using OAuth (API key mode). With OAuth we use our own REST client and Bearer token.

---

## Optional: refresh token (Connect API)

For **end-user** OAuth (Connect API), Alpaca can issue a refresh token. This project’s client credentials flow does **not** use refresh tokens; we only use `client_id` + `client_secret` → access token. If you later integrate the Connect API flow (user authorizes in browser), you would store `refresh_token` and use it to get new access tokens; env vars `ALPACA_REFRESH_TOKEN` and `OP_ALPACA_REFRESH_TOKEN_SECRET` exist in the client for that possibility.

---

## Summary

| Goal | Use |
|------|-----|
| Authenticate this app with Alpaca (no end-user login) | OAuth **client credentials**: register OAuth app → set `ALPACA_CLIENT_ID` + `ALPACA_CLIENT_SECRET` (or 1Password refs). |
| Let end users connect their Alpaca accounts to your app | Alpaca **Connect API** (authorization code flow); not implemented in this repo. |
| Simple per-account keys from dashboard | **API keys**: `ALPACA_API_KEY_ID` + `ALPACA_API_SECRET_KEY`. |

See also: [ALPACA_SECRETS_SETUP.md](ALPACA_SECRETS_SETUP.md), [ONEPASSWORD_INTEGRATION.md](ONEPASSWORD_INTEGRATION.md). Alpaca/Tastytrade runtime is retired from the active daemon set; see [BACKEND_SERVICES_DAEMONIZED.md](BACKEND_SERVICES_DAEMONIZED.md).
