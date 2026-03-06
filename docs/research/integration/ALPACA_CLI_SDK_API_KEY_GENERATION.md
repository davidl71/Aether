# Alpaca: CLI / SDK for Login and API Key Generation

**Question:** Does Alpaca provide a CLI, SDK, or example app to log in to the Alpaca site and generate or extract API keys programmatically?

**Answer: No.** Alpaca does **not** offer a public CLI or SDK that logs into alpaca.markets and creates or retrieves Trading API keys (Key ID + Secret) for you.

## How keys are created today

- **Trading API (personal/business):** API keys are created only in the **web dashboard**:
  - Sign in at [app.alpaca.markets](https://app.alpaca.markets/)
  - Paper → API Keys (or Live → API Keys) → Generate Key
  - Copy Key ID and Secret manually; there is no API or CLI to create or list these keys.
- **OAuth (Connect API):** For **apps** that let end users connect their Alpaca accounts:
  - You **register your app** in the dashboard (OAuth Apps) and get **client_id** and **client_secret**.
  - End users authorize via browser; your app receives OAuth tokens to act on their behalf.
  - This does not create or expose “API keys” for the developer; it’s an OAuth flow for end-user accounts.
- **Broker API:** For building your own brokerage (account opening, etc.); not used for generating personal Trading API keys.

## What Alpaca does provide

- **SDKs** (e.g. [alpaca-py](https://github.com/alpacahq/alpaca-py)) to **use** API keys (env vars or config) to call the Trading/Market Data APIs. They assume you already have keys from the dashboard.
- **OAuth integration guide** for building apps that connect end users’ Alpaca accounts (client credentials + OAuth flow), not for generating your own API keys.
- **Dashboard-only** creation of Paper/Live API keys; no documented “create key” or “list keys” REST endpoint for retail/business Trading API.

## Implications for this project

- **Secrets setup** must be manual or via a secret manager (e.g. 1Password) that stores keys you create in the dashboard.
- We cannot automate “login to Alpaca and generate API key” in a script without:
  - Browser automation (fragile, against ToS risk), or
  - An official Alpaca “developer API” for key creation (not currently public).

## References

- [Alpaca docs](https://docs.alpaca.markets/) – Trading API, Broker API, Market Data, OAuth.
- [Getting started with Trading API](https://docs.alpaca.markets/docs/getting-started-with-trading-api) – “create a free alpaca account, locate your API keys.”
- [About Connect API (OAuth)](https://docs.alpaca.markets/docs/about-connect-api) – OAuth for end-user app connections.
- [alpaca-py](https://github.com/alpacahq/alpaca-py) – Python SDK; examples assume keys from env/dashboard.

*Checked: 2025-03; Alpaca docs and public SDKs as of then.*
