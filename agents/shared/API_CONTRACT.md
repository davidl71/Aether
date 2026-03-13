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

## Loans API

### Loan Types (Enum)

| Value | Description |
|-------|-------------|
| `SHIR_BASED` | Standard SHIR-based loan (canonical) |
| `CPI_LINKED` | CPI-linked loan with inflation adjustment |

**Legacy Alias**: `SHIR` is accepted and maps to `SHIR_BASED` for backward compatibility.

### Loan Status (Enum)

| Value | Description |
|-------|-------------|
| `ACTIVE` | Loan is currently active |
| `PAID_OFF` | Loan has been fully repaid |
| `DEFAULTED` | Loan is in default |

### Endpoints

#### GET /api/v1/loans

List all loans.

**Response**:
```json
{
  "loans": [
    {
      "loan_id": "loan-1",
      "bank_name": "Discount",
      "account_number": "123456789",
      "loan_type": "SHIR_BASED",
      "principal": 1000.0,
      "original_principal": 1200.0,
      "interest_rate": 4.0,
      "spread": 0.5,
      "base_cpi": 0.0,
      "current_cpi": 0.0,
      "origination_date": "2025-01-01T00:00:00Z",
      "maturity_date": "2030-01-01T00:00:00Z",
      "next_payment_date": "2025-02-01T00:00:00Z",
      "monthly_payment": 100.0,
      "payment_frequency_months": 1,
      "status": "ACTIVE",
      "last_update": "2025-01-15T00:00:00Z"
    }
  ]
}
```

#### GET /api/v1/loans/:loan_id

Get a single loan by ID.

**Response**: `LoanRecord` object (see above).

#### POST /api/v1/loans

Create a new loan.

**Request**:
```json
{
  "loan_id": "loan-new",
  "bank_name": "Discount",
  "account_number": "987654321",
  "loan_type": "CPI_LINKED",
  "principal": 5000.0,
  "original_principal": 5000.0,
  "interest_rate": 3.5,
  "spread": 0.25,
  "base_cpi": 250.0,
  "current_cpi": 255.0,
  "origination_date": "2025-03-01T00:00:00Z",
  "maturity_date": "2032-03-01T00:00:00Z",
  "next_payment_date": "2025-04-01T00:00:00Z",
  "monthly_payment": 150.0,
  "payment_frequency_months": 1,
  "status": "ACTIVE",
  "last_update": "2025-03-01T00:00:00Z"
}
```

**Response**: `201 Created` with the created `LoanRecord`.

#### PUT /api/v1/loans/:loan_id

Update an existing loan.

**Request**: `LoanRecord` object.

**Response**: Updated `LoanRecord`.

#### DELETE /api/v1/loans/:loan_id

Delete a loan.

**Response**: `204 No Content` on success, `404 Not Found` if loan doesn't exist.

### JSON File Format

Loans can also be loaded from `config/loans.json`. The file should contain an array of `LoanRecord` objects.

**Canonical format**:
```json
[
  {
    "loan_id": "loan-1",
    "bank_name": "Discount",
    "account_number": "123456789",
    "loan_type": "SHIR_BASED",
    "principal": 1000.0,
    "original_principal": 1200.0,
    "interest_rate": 4.0,
    "spread": 0.5,
    "base_cpi": 0.0,
    "current_cpi": 0.0,
    "origination_date": "2025-01-01T00:00:00Z",
    "maturity_date": "2030-01-01T00:00:00Z",
    "next_payment_date": "2025-02-01T00:00:00Z",
    "monthly_payment": 100.0,
    "payment_frequency_months": 1,
    "status": "ACTIVE",
    "last_update": "2025-01-15T00:00:00Z"
  }
]
```

**Legacy format** (auto-converted to canonical):
```json
[
  {
    "loan_id": "loan-legacy",
    "bank_name": "Discount",
    "account_number": "123456789",
    "loan_type": "SHIR",
    "principal": 1000.0,
    "original_principal": 1000.0,
    "interest_rate": 4.0,
    "spread": 0.5,
    "base_cpi": 0.0,
    "current_cpi": 0.0,
    "origination_date": "2025-01-01T00:00:00Z",
    "maturity_date": "2030-01-01T00:00:00Z",
    "next_payment_date": "2025-02-01T00:00:00Z",
    "monthly_payment": 100.0,
    "payment_frequency_months": 1,
    "status": "ACTIVE",
    "last_update": "2025-01-15T00:00:00Z"
  }
]
```

The legacy `loan_type: "SHIR"` is automatically converted to `SHIR_BASED` for compatibility with newer clients.
