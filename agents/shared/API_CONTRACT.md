# Shared API Contract

This document captures the REST/WebSocket schema shared by the backend, Python/Textual TUI, and web client.

## Snapshot Endpoint

- **Method**: `GET /api/v1/snapshot`
- **Response**: `application/json`

```jsonc
{
  "generated_at": "2025-11-07T10:00:00Z",
  "mode": "DRY-RUN",
  "strategy": "RUNNING",
  "account_id": "DU123456",
  "metrics": {
    "net_liq": 100523.45,
    "buying_power": 80412.33,
    "excess_liquidity": 25000.00,
    "margin_requirement": 15000.00,
    "commissions": 123.45,
    "portal_ok": true,
    "tws_ok": true,
    "orats_ok": true,
    "questdb_ok": true
  },
  "symbols": [
    {
      "symbol": "SPY",
      "last": 509.2,
      "bid": 509.15,
      "ask": 509.18,
      "spread": 0.03,
      "roi": 0.65,
      "maker_count": 1,
      "taker_count": 0,
      "volume": 120,
      "candle": {
        "open": 509.0,
        "high": 509.4,
        "low": 508.6,
        "close": 509.2,
        "volume": 1500,
        "entry": 508.9,
        "updated": "2025-11-07T09:59:30Z"
      }
    }
  ],
  "positions": [],
  "historic": [],
  "orders": [],
  "decisions": [
    {
      "symbol": "SPY",
      "quantity": 1,
      "side": "BUY",
      "mark": 509.18,
      "created_at": "2025-11-07T10:00:05Z"
    }
  ],
  "alerts": [],
  "risk": {
    "allowed": true,
    "reason": null,
    "updated_at": "2025-11-07T10:00:05Z"
  }
}
```

> Update this contract whenever the backend changes payload fields. Frontend/TUI agents should sync against this spec.

**Livevol Integration Note**
- When Livevol credentials (`LIVEVOL_API_KEY`, `LIVEVOL_API_SECRET`) are present, the backend should enrich `symbols[].candle` and `positions[]` data with Cboe strategy quotes.
- Frontends/TUI treat these the same as IB-derived quotes; the source is transparent in the payload.

## Command Endpoints (WIP)

- `POST /api/v1/strategy/start`
- `POST /api/v1/strategy/stop`
- `POST /api/v1/orders/cancel`
- `POST /api/v1/combos/buy`
- `POST /api/v1/combos/sell`

Define request/response schemas as the backend endpoints solidify.
