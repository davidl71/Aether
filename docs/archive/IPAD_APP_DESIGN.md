# iPad Frontend Architecture

> **Note:** This design document is preserved for future implementation reference. iPad app development has been deferred in favor of focusing on PWA functionality. This documentation remains available for when iPad app development resumes.

## Goals

- Provide live visibility into account health, strategy status, positions, and orders.
- Offer light control capabilities (strategy start/stop, staged order overrides) with safety checks.
- Reuse existing integrations: TWS (execution), Nautilus strategy runner, ORATS enrichment, Client Portal/Web API snapshots, QuestDB historical data.
- Deliver a native SwiftUI iPad experience with responsive layouts and offline resilience.

## High-Level Components

1. **Backend Service Layer** (existing Python integration, extended in TODO #7)
   - REST API (FastAPI/Flask) hosting JSON endpoints for the app.
   - WebSocket channel for real-time pushes (optional phase 2).
   - Authentication middleware (initially static token; future SSO).
   - Command dispatcher for start/stop/cancel and dry-run toggles.

2. **Data Sources**
   - **Client Portal API**: account summary, portfolio, cash balances.
   - **Strategy Runner**: current state, metrics, pending orders (Python integration).
   - **QuestDB**: historical PnL, exposure, liquidity metrics.
   - **Notification Center**: events surfaced to app via /events endpoint.

3. **SwiftUI iPad App**
   - MVVM-style architecture with `ObservableObject` view models.
   - Secure token storage via Keychain.
   - Background refresh + manual pull-to-refresh.

## API Contract (Draft)

| Endpoint | Method | Auth | Description |
| --- | --- | --- | --- |
| `/api/v1/health` | GET | Optional | Backend status, build info, environment |
| `/api/v1/account/summary` | GET | Token | Client Portal summary, buying power, net liq |
| `/api/v1/account/positions` | GET | Token | Consolidated positions (Portal + strategy view) |
| `/api/v1/strategy/status` | GET | Token | Running state, symbols, timestamps, risk metrics |
| `/api/v1/strategy/stats` | GET | Token | Computed metrics (ROI, arbitrage wins/losses) |
| `/api/v1/orders/recent` | GET | Token | Recent orders & fills from Portal/strategy |
| `/api/v1/orders/cancel` | POST | Token + Conf | Cancel specified order/box spread |
| `/api/v1/strategy/start` | POST | Token + Conf | Start strategy loop (with dry-run flag) |
| `/api/v1/strategy/stop` | POST | Token + Conf | Graceful stop |
| `/api/v1/events` | GET | Token | Paginated feed of notifications |
| `/api/v1/history/pnl` | GET | Token | QuestDB aggregated PnL data |

### Command Safeguards

- Commands require a confirmation payload `{ "confirm": true }`.
- Optional OTP second-factor for live trading.
- Backend logs every command with user/device info.

## App Navigation

1. **Onboarding/Login**
   - Server URL, API token, optional TouchID/FaceID.
2. **Dashboard Tab**
   - Cards: Net Liquidity, Buying Power, Strategy Status.
   - Inline alerts pulled from `/events`.
3. **Positions Tab**
   - Grid/List of positions with filters (strategy vs. manual).
   - Detail sheets showing ORATS data, IV percentiles, risk flags.
4. **Orders Tab**
   - Timeline of orders/fills with statuses.
   - Quick access to cancel pending box spreads.
5. **Controls Tab**
   - Toggles and action buttons with safety prompts.
   - Dry-run toggle, re-auth status, QuestDB sync indicator.
6. **Settings Tab**
   - Environment config, theme, notification preferences, logs.

## Data Flow

```
iPad App (SwiftUI)
  ↕ HTTPS (REST/JSON)
Backend API (Python FastAPI)
  ↕ Nautilus Strategy Runner (in-process calls)
  ↕ Client Portal API (requests)
  ↕ QuestDB (HTTP ILP/REST)
  ↕ Notification Center (internal queue)
```

## Security & Deployment

- Initial auth: static API token stored in Keychain + HTTPS only.
- Recommend reverse proxy (nginx) terminating TLS with client certs.
- Stage environment for testing with paper trading + sandbox QuestDB.
- Apple Developer setup: app identifier, profiles, TestFlight distribution.

## Milestones

1. Finalize API schema and backend tasks (TODO #7).
2. Build SwiftUI scaffold and integrate login + dashboard skeleton (TODO #8).
3. Implement portfolio/order views using live endpoints (TODO #9).
4. Add strategy controls with confirmation UX and automated tests (TODO #10).
5. QA, TestFlight, and release readiness checklist.
