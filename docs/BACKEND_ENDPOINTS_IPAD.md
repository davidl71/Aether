# Backend Endpoints for iPad App

**Date**: 2025-11-22
**Status**: Design Complete
**Task**: T-216 (Backend endpoints for iPad app)

## Overview

Comprehensive REST API endpoint specification for iPad app, extending existing Rust backend REST API to support all iPad app features.

## Current Backend State

**Existing Endpoints** (from `agents/backend/crates/api/src/rest.rs`):

- ✅ `GET /health` - Health check with component status
- ✅ `GET /api/v1/snapshot` - Complete system snapshot
- ✅ `POST /api/v1/strategy/start` - Start strategy
- ✅ `POST /api/v1/strategy/stop` - Stop strategy
- ✅ `GET /api/v1/strategy/status` - Get strategy status
- ✅ `GET /api/v1/orders` - List orders
- ✅ `POST /api/v1/orders/cancel` - Cancel order
- ✅ `GET /api/v1/orders/:order_id` - Get order details
- ✅ `POST /api/mode` - Toggle mode (dry-run/live)
- ✅ `POST /api/account` - Change account
- ✅ `GET /api/v1/config` - Get configuration
- ✅ `PUT /api/v1/config` - Update configuration
- ✅ `GET /api/v1/scenarios` - Get box spread scenarios
- ✅ Swiftness endpoints (positions, portfolio-value, validate, exchange-rate)

## Required Endpoints for iPad

### 1. Account Information

#### Get Account Summary


```
GET /api/v1/account/summary
Authorization: Bearer <token>

Response:
{
  "account_id": "DU123456",
  "net_liquidation": 100523.45,
  "buying_power": 80412.33,
  "excess_liquidity": 25000.00,
  "margin_requirement": 15000.00,
  "cash_balance": 50000.00,
  "equity": 100523.45,
  "last_update": "2025-11-22T10:00:00Z"
}
```

**Implementation**: Extend existing snapshot endpoint or create dedicated account summary endpoint.


#### Get Account Positions

```
GET /api/v1/account/positions
Authorization: Bearer <token>
Query Params: ?type=current|historic&symbol=SPX

Response:
{
  "positions": [
    {
      "id": "pos_123",
      "symbol": "SPX",
      "quantity": 1,
      "avg_price": 509.50,
      "current_price": 510.25,
      "unrealized_pnl": 0.75,
      "realized_pnl": 0.0,
      "entry_time": "2025-11-22T09:00:00Z",
      "last_update": "2025-11-22T10:00:00Z"
    }
  ]
}
```

**Implementation**: Extract from existing snapshot endpoint or create dedicated positions endpoint.

### 2. Strategy Information


#### Get Strategy Statistics

```
GET /api/v1/strategy/stats
Authorization: Bearer <token>

Response:
{
  "total_trades": 45,
  "winning_trades": 38,
  "losing_trades": 7,
  "total_profit": 1234.56,
  "total_loss": -234.12,
  "net_pnl": 1000.44,
  "win_rate": 0.844,
  "avg_profit_per_trade": 27.43,
  "avg_loss_per_trade": -33.45,
  "largest_win": 125.00,
  "largest_loss": -50.00,
  "period_start": "2025-11-01T00:00:00Z",
  "period_end": "2025-11-22T10:00:00Z"
}
```

**Implementation**: New endpoint - aggregate data from QuestDB or strategy runner.


### 3. Historical Data

#### Get Historical PnL

```
GET /api/v1/history/pnl
Authorization: Bearer <token>
Query Params: ?start_date=2025-11-01&end_date=2025-11-22&granularity=daily|hourly

Response:
{
  "data": [
    {
      "date": "2025-11-22",
      "pnl": 45.67,
      "cumulative_pnl": 1000.44,
      "trades": 3
    }
  ],
  "period": {
    "start": "2025-11-01T00:00:00Z",
    "end": "2025-11-22T23:59:59Z"
  }
}
```

**Implementation**: Query QuestDB for historical PnL data.


### 4. Events/Notifications

#### Get Events Feed

```
GET /api/v1/events
Authorization: Bearer <token>
Query Params: ?limit=50&offset=0&severity=info|warning|error

Response:
{
  "events": [
    {
      "id": "evt_123",
      "type": "order_filled",
      "severity": "info",
      "message": "Order 12345 filled at 509.50",
      "timestamp": "2025-11-22T10:00:00Z",
      "metadata": {
        "order_id": "12345",
        "symbol": "SPX",
        "fill_price": 509.50
      }
    }
  ],
  "pagination": {
    "total": 150,
    "limit": 50,
    "offset": 0,
    "has_more": true
  }
}
```


**Implementation**: Query from alerts/notifications system or NATS message history.

### 5. Enhanced Order Management

#### Get Recent Orders

```
GET /api/v1/orders/recent
Authorization: Bearer <token>
Query Params: ?limit=100&status=pending|filled|cancelled

Response:
{
  "orders": [
    {
      "id": "12345",
      "symbol": "SPX",
      "quantity": 1,
      "action": "BUY",
      "status": "FILLED",
      "limit_price": 509.50,
      "filled_quantity": 1,
      "avg_fill_price": 509.50,
      "created_at": "2025-11-22T10:00:00Z",
      "filled_at": "2025-11-22T10:00:05Z"
    }
  ]
}
```

**Implementation**: Extend existing `/api/v1/orders` endpoint with filtering.

## Implementation Plan

### Phase 1: Extend Existing Endpoints

**File**: `agents/backend/crates/api/src/rest.rs`

1. **Enhance `/api/v1/snapshot`**:
   - Already provides comprehensive data
   - iPad can use this as primary data source
   - No changes needed

2. **Enhance `/api/v1/orders`**:
   - Add filtering by status
   - Add date range filtering
   - Add sorting options

3. **Enhance `/api/v1/strategy/status`**:
   - Add statistics (win rate, total trades, etc.)
   - Add performance metrics

### Phase 2: Add New Endpoints

1. **`GET /api/v1/account/summary`**:
   - Extract account metrics from snapshot
   - Lightweight endpoint for quick status checks

2. **`GET /api/v1/strategy/stats`**:
   - Aggregate statistics from QuestDB
   - Performance metrics calculation

3. **`GET /api/v1/history/pnl`**:
   - Query QuestDB for historical PnL
   - Support different granularities

4. **`GET /api/v1/events`**:
   - Query alerts/notifications
   - Pagination support

### Phase 3: WebSocket Support (Future)

**Endpoint**: `WS /api/v1/stream`

- Real-time snapshot updates
- Order status changes

- Strategy status changes
- Alert notifications

## Request/Response Schemas

### Common Request Headers


```
Authorization: Bearer <token>
Content-Type: application/json
Accept: application/json
```

### Common Response Format

```json
{

  "status": "ok" | "error",
  "data": { ... },
  "message": "Success message",
  "timestamp": "2025-11-22T10:00:00Z"
}
```

### Error Response Format

```json
{
  "status": "error",
  "error": {
    "code": "ORDER_NOT_FOUND",
    "message": "Order with ID 12345 not found",

    "details": {}
  },
  "timestamp": "2025-11-22T10:00:00Z"
}

```

## Authentication

### Current Implementation

- Token-based authentication (Bearer token)

- Token passed in `Authorization` header

### Future Enhancements

- OAuth 2.0 support
- Refresh token mechanism
- Biometric authentication integration


## Rate Limiting

### Recommendations


- 100 requests per minute per client
- Burst allowance: 20 requests per 10 seconds
- WebSocket connections: 1 per client

## Caching Strategy

### Client-Side (iPad)


- Cache snapshot for offline access
- Cache configuration
- Cache historical data (with TTL)


### Server-Side

- Cache account summary (30 seconds)
- Cache strategy stats (60 seconds)
- Cache scenarios (5 minutes)


## Testing

### Unit Tests

- Test each endpoint handler
- Test request validation

- Test error cases

### Integration Tests

- Test with real backend

- Test authentication flow
- Test error scenarios

### End-to-End Tests

- Test complete user flows
- Test offline/online transitions
- Test error recovery

## Documentation

### OpenAPI/Swagger

- Auto-generate from Rust code
- Interactive API documentation
- Request/response examples

### Location

- `/docs` - Swagger UI
- `/redoc` - ReDoc

## Success Criteria

- [ ] All required endpoints implemented
- [ ] Authentication working
- [ ] Error handling comprehensive
- [ ] Rate limiting configured
- [ ] Caching strategy implemented
- [ ] Tests passing
- [ ] Documentation complete
- [ ] Performance acceptable

## Related Documentation

- [REST API Layer Design](research/architecture/REST_API_LAYER_DESIGN.md)
- [Shared API Contract](../agents/shared/API_CONTRACT.md)
- [iPad Frontend Architecture](IPAD_FRONTEND_ARCHITECTURE.md)
