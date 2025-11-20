# REST API Layer Design for Web SPA

**Date**: 2025-11-17
**Status**: Design Document
**Purpose**: Design specification for implementing REST API layer for web SPA (Agent TODO #12)

---

## Overview

Design and implement comprehensive REST API endpoints to support web SPA functionality, including strategy control, order management, and configuration.

---

## Current State

**Existing Endpoints:**
- `GET /api/snapshot` - Snapshot data (Python FastAPI)
- `GET /api/health` - Health check (Rust backend)
- `GET /api/v1/snapshot` - Snapshot data (Rust backend)

**Web App Needs:**
- Strategy start/stop
- Order cancellation
- Mode switching (dry-run/live)
- Account selection
- Real-time updates (WebSocket - separate task)

---

## API Endpoints

### 1. Strategy Control

**Start Strategy:**
```
POST /api/v1/strategy/start
Request: {}
Response: {
  "status": "ok",
  "message": "Strategy started",
  "strategy_status": "RUNNING"
}
```

**Stop Strategy:**
```
POST /api/v1/strategy/stop
Request: {}
Response: {
  "status": "ok",
  "message": "Strategy stopped",
  "strategy_status": "STOPPED"
}
```

**Get Strategy Status:**
```
GET /api/v1/strategy/status
Response: {
  "status": "RUNNING" | "STOPPED" | "ERROR",
  "started_at": "2025-11-17T10:00:00Z",
  "last_update": "2025-11-17T10:05:00Z"
}
```

---

### 2. Order Management

**Cancel Order:**
```
POST /api/v1/orders/cancel
Request: {
  "order_id": "12345"
}
Response: {
  "status": "ok",
  "message": "Order cancelled",
  "order_id": "12345"
}
```

**Get Orders:**
```
GET /api/v1/orders
Query Params: ?status=pending&limit=100
Response: {
  "orders": [
    {
      "order_id": "12345",
      "symbol": "SPX",
      "quantity": 1,
      "status": "PENDING",
      "created_at": "2025-11-17T10:00:00Z"
    }
  ]
}
```

**Get Order Details:**
```
GET /api/v1/orders/{order_id}
Response: {
  "order_id": "12345",
  "symbol": "SPX",
  "quantity": 1,
  "status": "FILLED",
  "fill_price": 509.50,
  "created_at": "2025-11-17T10:00:00Z",
  "filled_at": "2025-11-17T10:00:05Z"
}
```

---

### 3. Mode & Configuration

**Toggle Mode:**
```
POST /api/mode
Request: {
  "mode": "DRY-RUN" | "LIVE"
}
Response: {
  "status": "ok",
  "message": "Mode changed to DRY-RUN",
  "mode": "DRY-RUN"
}
```

**Change Account:**
```
POST /api/account
Request: {
  "account_id": "DU123456"
}
Response: {
  "status": "ok",
  "message": "Account changed",
  "account_id": "DU123456"
}
```

**Get Configuration:**
```
GET /api/v1/config
Response: {
  "mode": "DRY-RUN",
  "strategy": {
    "symbols": ["SPX", "XSP"],
    "min_arbitrage_profit": 0.25
  },
  "risk": {
    "max_total_exposure": 100000.0
  }
}
```

**Update Configuration:**
```
PUT /api/v1/config
Request: {
  "strategy": {
    "min_arbitrage_profit": 0.30
  }
}
Response: {
  "status": "ok",
  "message": "Configuration updated"
}
```

---

### 4. Box Spread Scenarios

**Get Scenarios:**
```
GET /api/v1/scenarios
Query Params: ?symbol=SPX&min_apr=10.0
Response: {
  "scenarios": [
    {
      "symbol": "SPX",
      "expiration": "2025-12-19",
      "strike_width": 100,
      "net_debit": 95.50,
      "profit": 4.50,
      "roi": 4.7,
      "annualized_return": 12.5,
      "fill_probability": 0.85
    }
  ],
  "as_of": "2025-11-17T10:00:00Z",
  "underlying": "SPX"
}
```

---

## Implementation

### Python FastAPI Service

**File**: `python/integration/alpaca_service.py` (extend existing)

```python
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel

app = FastAPI()

class StrategyStartRequest(BaseModel):
    pass

class StrategyStopRequest(BaseModel):
    pass

class CancelOrderRequest(BaseModel):
    order_id: str

class ModeRequest(BaseModel):
    mode: str

@app.post("/api/v1/strategy/start")
async def start_strategy(request: StrategyStartRequest):
    # Start strategy logic
    return {"status": "ok", "message": "Strategy started"}

@app.post("/api/v1/strategy/stop")
async def stop_strategy(request: StrategyStopRequest):
    # Stop strategy logic
    return {"status": "ok", "message": "Strategy stopped"}

@app.post("/api/v1/orders/cancel")
async def cancel_order(request: CancelOrderRequest):
    # Cancel order logic
    return {"status": "ok", "message": "Order cancelled"}
```

---

### Rust Backend Service

**File**: `agents/backend/crates/api/src/rest.rs` (extend existing)

```rust
use axum::{Json, Router, routing::post};

pub fn router(state: RestState) -> Router<RestState> {
    Router::new()
        .route("/api/v1/strategy/start", post(strategy_start))
        .route("/api/v1/strategy/stop", post(strategy_stop))
        .route("/api/v1/orders/cancel", post(cancel_order))
        .with_state(state)
}

async fn strategy_start(State(state): State<RestState>) -> Json<serde_json::Value> {
    // Start strategy
    Json(json!({"status": "ok", "message": "Strategy started"}))
}
```

---

## Error Handling

**Standard Error Response:**
```json
{
  "status": "error",
  "message": "Order not found",
  "error_code": "ORDER_NOT_FOUND",
  "details": {}
}
```

**HTTP Status Codes:**
- `200 OK` - Success
- `400 Bad Request` - Invalid request
- `404 Not Found` - Resource not found
- `500 Internal Server Error` - Server error

---

## Authentication & Security

**Authentication:**
- API key in header: `X-API-Key: <key>`
- Or token-based: `Authorization: Bearer <token>`
- Rate limiting per client

**Security:**
- HTTPS in production
- Input validation
- SQL injection prevention
- XSS prevention

---

## API Versioning

**Version Strategy:**
- `/api/v1/` - Current version
- `/api/v2/` - Future version
- Backward compatibility maintained

---

## Documentation

**OpenAPI/Swagger:**
- Auto-generate from FastAPI
- Interactive API documentation
- Request/response examples

**Location:**
- `/docs` - Swagger UI
- `/redoc` - ReDoc

---

## Testing

**Unit Tests:**
- Test each endpoint
- Test error cases
- Test validation

**Integration Tests:**
- Test with real backend
- Test end-to-end flows
- Test error scenarios

---

## Implementation Steps

1. **Extend FastAPI Service**
   - Add strategy control endpoints
   - Add order management endpoints
   - Add mode/account endpoints
   - Add error handling

2. **Extend Rust Backend**
   - Add REST endpoints
   - Integrate with strategy engine
   - Add order management

3. **Update API Contract**
   - Document all endpoints
   - Update `API_CONTRACT.md`
   - Add request/response schemas

4. **Testing**
   - Unit tests
   - Integration tests
   - End-to-end tests

5. **Documentation**
   - OpenAPI spec
   - Usage examples
   - Error handling guide

---

## Success Criteria

- [ ] All endpoints implemented
- [ ] Error handling comprehensive
- [ ] Authentication/authorization
- [ ] API documentation complete
- [ ] Tests passing
- [ ] Performance acceptable
- [ ] Security measures in place

---

**Document Status**: ✅ Complete - Design specification ready for implementation
