# iPad Frontend Architecture

> Historical reference only. Apple-client work was removed from the active platform scope on 2026-03-10. This document is preserved only as deferred design material and is not part of the current frontend plan.

**Date**: 2025-11-22
**Status**: Deferred / Historical
**Task**: T-215 (historical iPad frontend architecture design)

## Overview

Comprehensive SwiftUI architecture design for native iPad trading application, providing live visibility into account health, strategy status, positions, and orders with light control capabilities.

## Goals

- Provide live visibility into account health, strategy status, positions, and orders
- Offer light control capabilities (strategy start/stop, staged order overrides) with safety checks
- Reuse existing integrations: TWS (execution), Rust backend REST API, ORATS enrichment, QuestDB historical data
- Deliver a native SwiftUI iPad experience with responsive layouts and offline resilience
- Align with web/TUI feature parity

## Architecture Pattern

### MVVM (Model-View-ViewModel)

```
┌─────────────────────────────────────────────────────────┐
│                    SwiftUI Views                        │
│  (DashboardView, PositionsView, OrdersView, etc.)      │
└──────────────────────┬──────────────────────────────────┘
                       │ @ObservedObject / @StateObject
┌──────────────────────▼──────────────────────────────────┐
│                  ViewModels                              │
│  (DashboardViewModel, PositionsViewModel, etc.)        │
│  - ObservableObject                                      │
│  - Business logic                                        │
│  - State management                                      │
└──────────────────────┬──────────────────────────────────┘
                       │ API calls
┌──────────────────────▼──────────────────────────────────┐
│                  Services Layer                         │
│  (APIClient, AuthService, DataService)                 │
│  - REST API communication                               │
│  - WebSocket connection (future)                        │
│  - Data caching                                         │
└──────────────────────┬──────────────────────────────────┘
                       │ HTTPS / WebSocket
┌──────────────────────▼──────────────────────────────────┐
│              Rust Backend REST API                      │
│  (agents/backend/crates/api/src/rest.rs)               │
└─────────────────────────────────────────────────────────┘
```

## Component Structure

### 1. App Entry Point

**File**: `ios/BoxSpreadIPad/BoxSpreadIPadApp.swift`

```swift
@main
struct BoxSpreadIPadApp: App {
    @StateObject private var appState = AppState()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(appState)
        }
    }
}
```

### 2. Core Views

#### DashboardView

- **Purpose**: Main overview with account metrics, strategy status, alerts
- **Components**:
  - AccountSummaryCard
  - StrategyStatusCard
  - AlertsFeed
  - QuickActionsPanel

#### PositionsView

- **Purpose**: Display current and historic positions
- **Components**:
  - PositionsList
  - PositionDetailSheet
  - Filters (strategy vs manual, symbol, date range)

#### OrdersView

- **Purpose**: Timeline of orders and fills
- **Components**:
  - OrdersTimeline
  - OrderDetailSheet
  - CancelOrderButton (with confirmation)

#### ControlsView

- **Purpose**: Strategy control and configuration
- **Components**:
  - StrategyStartStopButton
  - DryRunToggle
  - AccountSelector
  - RiskSettingsPanel

#### SettingsView

- **Purpose**: App configuration and preferences
- **Components**:
  - ServerConfiguration
  - ThemeSelector
  - NotificationPreferences
  - LogsViewer

### 3. ViewModels

#### DashboardViewModel

```swift
@MainActor
class DashboardViewModel: ObservableObject {
    @Published var snapshot: SnapshotPayload?
    @Published var isLoading = false
    @Published var error: AppError?

    private let apiClient: APIClient

    func refresh() async {
        // Fetch snapshot data
    }

    func startStrategy() async {
        // Start strategy via API
    }

    func stopStrategy() async {
        // Stop strategy via API
    }
}
```

#### PositionsViewModel

```swift
@MainActor
class PositionsViewModel: ObservableObject {
    @Published var positions: [PositionSnapshot] = []
    @Published var historicPositions: [PositionSnapshot] = []
    @Published var selectedPosition: PositionSnapshot?

    func loadPositions() async {
        // Load from snapshot
    }

    func loadHistoric() async {
        // Load historic positions
    }
}
```

#### OrdersViewModel

```swift
@MainActor
class OrdersViewModel: ObservableObject {
    @Published var orders: [OrderSnapshot] = []
    @Published var selectedOrder: OrderSnapshot?

    func loadOrders() async {
        // Load orders from snapshot
    }

    func cancelOrder(_ orderId: String) async throws {
        // Cancel order via API
    }
}
```

### 4. Services Layer

#### APIClient

```swift
class APIClient {
    private let baseURL: URL
    private let session: URLSession
    private let authToken: String

    func fetchSnapshot() async throws -> SnapshotPayload {
        // GET /api/v1/snapshot
    }

    func startStrategy() async throws -> StrategyResponse {
        // POST /api/v1/strategy/start
    }

    func stopStrategy() async throws -> StrategyResponse {
        // POST /api/v1/strategy/stop
    }

    func cancelOrder(_ orderId: String) async throws -> CancelResponse {
        // POST /api/v1/orders/cancel
    }

    func toggleMode(_ mode: TradingMode) async throws -> ModeResponse {
        // POST /api/mode
    }
}
```

#### AuthService

```swift
class AuthService {
    private let keychain: KeychainService

    func storeToken(_ token: String) {
        // Store in Keychain
    }

    func retrieveToken() -> String? {
        // Retrieve from Keychain
    }

    func authenticate(serverURL: String, token: String) async throws {
        // Validate credentials
    }
}
```

#### DataService

```swift
class DataService {
    private let cache: NSCache<NSString, SnapshotPayload>

    func cacheSnapshot(_ snapshot: SnapshotPayload) {
        // Cache for offline access
    }

    func getCachedSnapshot() -> SnapshotPayload? {
        // Retrieve cached data
    }
}
```

## Navigation Structure

### Tab-Based Navigation

```
┌─────────────────────────────────────────────────────────┐
│                    Tab Bar                              │
├─────────────────────────────────────────────────────────┤
│  Dashboard │ Positions │ Orders │ Controls │ Settings  │
└─────────────────────────────────────────────────────────┘
```

### Navigation Flow

```
OnboardingView
    ↓
ContentView (TabView)
    ├─ DashboardTab
    │   ├─ AccountSummaryCard
    │   ├─ StrategyStatusCard
    │   └─ AlertsFeed
    ├─ PositionsTab
    │   ├─ PositionsList
    │   └─ PositionDetailSheet (modal)
    ├─ OrdersTab
    │   ├─ OrdersTimeline
    │   └─ OrderDetailSheet (modal)
    ├─ ControlsTab
    │   ├─ StrategyControls
    │   └─ ConfigurationPanel
    └─ SettingsTab
        ├─ ServerConfig
        └─ Preferences
```

## Data Models

### SnapshotPayload

```swift
struct SnapshotPayload: Codable {
    let generatedAt: Date
    let mode: TradingMode
    let strategy: StrategyStatus
    let accountId: String
    let metrics: AccountMetrics
    let symbols: [SymbolSnapshot]
    let positions: [PositionSnapshot]
    let historic: [PositionSnapshot]
    let orders: [OrderSnapshot]
    let alerts: [Alert]
    let risk: RiskStatus
}
```

### AccountMetrics

```swift
struct AccountMetrics: Codable {
    let netLiq: Double
    let buyingPower: Double
    let excessLiquidity: Double
    let marginRequirement: Double
    let commissions: Double
    let portalOk: Bool
    let twsOk: Bool
    let oratsOk: Bool
    let questdbOk: Bool
    let natsOk: Bool
}
```

### PositionSnapshot

```swift
struct PositionSnapshot: Codable, Identifiable {
    let id: String
    let symbol: String
    let quantity: Int
    let avgPrice: Double
    let currentPrice: Double
    let unrealizedPnl: Double
    let realizedPnl: Double
    let entryTime: Date
    let lastUpdate: Date
}
```

### OrderSnapshot

```swift
struct OrderSnapshot: Codable, Identifiable {
    let id: String
    let symbol: String
    let quantity: Int
    let action: OrderAction
    let status: OrderStatus
    let limitPrice: Double?
    let filledQuantity: Int
    let avgFillPrice: Double?
    let createdAt: Date
    let lastUpdate: Date
}
```

## API Integration

### Endpoints Used

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/v1/snapshot` | GET | Main data source for all views |
| `/api/v1/strategy/start` | POST | Start strategy |
| `/api/v1/strategy/stop` | POST | Stop strategy |
| `/api/v1/strategy/status` | GET | Get strategy status |
| `/api/v1/orders` | GET | List orders |
| `/api/v1/orders/cancel` | POST | Cancel order |
| `/api/mode` | POST | Toggle dry-run/live mode |
| `/api/account` | POST | Change account |
| `/api/v1/config` | GET/PUT | Get/update configuration |
| `/api/v1/scenarios` | GET | Get box spread scenarios |
| `/health` | GET | Health check |

### Request/Response Examples

**Start Strategy:**

```swift
// Request
POST /api/v1/strategy/start
Headers: ["Authorization": "Bearer <token>"]

// Response
{
  "status": "ok",
  "message": "Strategy started",
  "strategy_status": "RUNNING"
}
```

**Cancel Order:**

```swift
// Request
POST /api/v1/orders/cancel
Body: { "order_id": "12345" }

// Response
{
  "status": "ok",
  "message": "Order cancelled",
  "order_id": "12345"
}
```

## UI Components

### Reusable Components

#### AccountSummaryCard

- Net Liquidity
- Buying Power
- Excess Liquidity
- Margin Requirement

#### StrategyStatusCard

- Strategy state (RUNNING/STOPPED)
- Mode indicator (DRY-RUN/LIVE)
- Last update timestamp
- Quick start/stop button

#### AlertsFeed

- Real-time alert stream
- Color-coded by severity
- Tap to view details

#### PositionsList

- Grid/List view toggle
- Filter by symbol, strategy, date
- Pull-to-refresh
- Swipe actions (view details, close position)

#### OrdersTimeline

- Chronological order list
- Status indicators
- Cancel action for pending orders
- Fill details for completed orders

## State Management

### AppState (Global)

```swift
@MainActor
class AppState: ObservableObject {
    @Published var isAuthenticated = false
    @Published var serverURL: String = ""
    @Published var currentAccount: String = ""
    @Published var theme: AppTheme = .dark
    @Published var refreshInterval: TimeInterval = 5.0
}
```

### View-Specific State

- Each view has its own ViewModel
- ViewModels are `@StateObject` in parent views
- Shared state passed via `@EnvironmentObject`

## Offline Support

### Caching Strategy

- Cache last snapshot in UserDefaults/NSCache
- Display cached data when offline
- Show offline indicator
- Queue actions for when connection restored

### Background Refresh

- Use `Task` with background refresh
- Refresh every 5 seconds when app is active
- Manual pull-to-refresh available

## Security

### Authentication

- API token stored in Keychain
- Biometric authentication (TouchID/FaceID) for sensitive actions
- Token refresh mechanism

### Secure Storage

```swift
class KeychainService {
    func store(_ value: String, forKey key: String) throws {
        // Store in Keychain
    }

    func retrieve(forKey key: String) throws -> String? {
        // Retrieve from Keychain
    }
}
```

### Network Security

- HTTPS only
- Certificate pinning (optional)
- Request signing (future)

## Error Handling

### Error Types

```swift
enum AppError: LocalizedError {
    case networkError(URLError)
    case apiError(statusCode: Int, message: String)
    case authenticationError
    case decodingError
    case unknownError

    var errorDescription: String? {
        // User-friendly error messages
    }
}
```

### Error Display

- Toast notifications for transient errors
- Alert dialogs for critical errors
- Inline error states in views

## Testing Strategy

### Unit Tests

- ViewModel logic
- API client parsing
- Data model validation

### UI Tests

- Navigation flows
- User interactions
- Error scenarios

### Integration Tests

- API communication
- Authentication flow
- Data persistence

## Performance Considerations

### Optimization

- Lazy loading for large lists
- Image caching
- Debounced API calls
- Efficient state updates

### Memory Management

- Weak references in closures
- Proper cleanup in view lifecycle
- Cache size limits

## Accessibility

### Support

- VoiceOver labels
- Dynamic Type support
- High contrast mode
- Reduced motion support

## Deployment

### Build Configuration

- Development: Local server
- Staging: Staging server
- Production: Production server

### Distribution

- TestFlight for beta testing
- App Store for production
- Enterprise distribution (if needed)

## Implementation Phases

### Phase 1: Foundation

1. SwiftUI project setup
2. Navigation structure
3. API client implementation
4. Authentication flow

### Phase 2: Core Views

1. Dashboard view
2. Positions view
3. Orders view
4. Settings view

### Phase 3: Controls

1. Strategy start/stop
2. Order cancellation
3. Mode switching
4. Account selection

### Phase 4: Polish

1. Offline support
2. Error handling
3. Performance optimization
4. Accessibility

## Success Criteria

- [ ] All views implemented and functional
- [ ] API integration complete
- [ ] Offline support working
- [ ] Error handling comprehensive
- [ ] Performance acceptable
- [ ] Accessibility standards met
- [ ] Tests passing
- [ ] Ready for TestFlight

## Related Documentation

- [REST API Layer Design](research/architecture/REST_API_LAYER_DESIGN.md)
- [Shared API Contract](../agents/shared/API_CONTRACT.md)
- [Web SPA Architecture](WEB_SPA_ARCHITECTURE.md)
