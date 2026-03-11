# Web SPA Architecture & Wireframes

**Date**: 2025-11-22
**Status**: Design Complete
**Task**: T-21 (Web SPA architecture/wireframes)

## Overview

Comprehensive architecture documentation and wireframes for the React/TypeScript Progressive Web App (PWA), documenting current implementation and future enhancements.

## Current Architecture

### Technology Stack

- **Framework**: React 18.3.1
- **Language**: TypeScript 5.5.4
- **Build Tool**: Vite 5.4.8
- **Styling**: CSS (dark trading desk palette)
- **State Management**: React Hooks (useState, useEffect, custom hooks)
- **Charts**: lightweight-charts 5.0.9
- **PWA**: vite-plugin-pwa 1.1.0

### Project Structure

```
web/
├── src/
│   ├── App.tsx                 # Main application component
│   ├── main.tsx                # Entry point
│   ├── components/             # UI components
│   │   ├── HeaderStatus.tsx
│   │   ├── TabNavigation.tsx
│   │   ├── DashboardTab.tsx
│   │   ├── PositionsTable.tsx
│   │   ├── OrdersPanel.tsx
│   │   ├── AlertsPanel.tsx
│   │   ├── BoxSpreadTable.tsx
│   │   ├── OptionsChainTable.tsx
│   │   ├── CandlestickChart.tsx
│   │   └── ...
│   ├── hooks/                 # Custom React hooks
│   │   ├── useSnapshot.ts
│   │   ├── useBoxSpreadData.ts
│   │   ├── useSymbolWatchlist.ts
│   │   ├── useChartData.ts
│   │   ├── useWebSocket.ts
│   │   └── ...
│   ├── api/                   # API clients
│   │   └── snapshot.ts
│   ├── types/                 # TypeScript types
│   │   ├── snapshot.ts
│   │   ├── chart.ts
│   │   └── types.ts
│   ├── utils/                 # Utility functions
│   │   ├── formatters.ts
│   │   └── volatility.ts
│   └── styles/
│       └── app.css            # Global styles
├── public/
│   ├── data/                  # Static data files
│   ├── icons/                 # PWA icons
│   └── manifest.json          # PWA manifest
└── package.json
```

## Component Architecture

### Component Hierarchy

```
App
├── HeaderStatus
│   ├── ModeSwitcher
│   ├── AccountSelector
│   └── StrategyControls
├── ScenarioSection
│   ├── ScenarioSummary
│   └── BoxSpreadTable
├── TabNavigation
└── Main Content (Tab-based)
    ├── DashboardTab
    │   ├── SymbolGrid
    │   ├── Sparkline
    │   └── QuickActions
    ├── PositionsTable (Current)
    ├── PositionsTable (Historic)
    ├── OrdersPanel
    │   └── OrderItem
    └── AlertsPanel
        └── AlertItem
└── DetailModal
    ├── SymbolDetail
    ├── PositionDetail
    └── ActionConfirmation
```

### Key Components

#### App.tsx

- **Purpose**: Main application container
- **State**: Active tab, modal state, snapshot data
- **Responsibilities**:
  - Data fetching coordination
  - Tab navigation
  - Modal management
  - API calls (strategy control, order cancellation)

#### HeaderStatus

- **Purpose**: System status banner
- **Displays**: Connection status, account info, mode, strategy status
- **Actions**: Mode toggle, account change, strategy start/stop

#### DashboardTab

- **Purpose**: Main overview
- **Displays**: Symbols, metrics, watchlist
- **Features**: Symbol filtering, watchlist management

#### PositionsTable

- **Purpose**: Display positions
- **Displays**: Current or historic positions
- **Features**: Sorting, filtering, detail view

#### OrdersPanel

- **Purpose**: Order timeline
- **Displays**: Orders with status
- **Features**: Cancel order action

#### BoxSpreadTable

- **Purpose**: Box spread scenarios
- **Displays**: Arbitrage opportunities
- **Features**: Sorting, filtering by APR/ROI

## Data Flow

### Data Fetching Pattern

```
┌─────────────────────────────────────────────────────────┐
│                    React Components                     │
└──────────────────────┬──────────────────────────────────┘
                       │ useSnapshot hook
┌──────────────────────▼──────────────────────────────────┐
│                  Custom Hooks                           │
│  (useSnapshot, useBoxSpreadData, useChartData)         │
└──────────────────────┬──────────────────────────────────┘
                       │ API calls
┌──────────────────────▼──────────────────────────────────┐
│                  API Clients                             │
│  (snapshot.ts)                                          │
└──────────────────────┬──────────────────────────────────┘
                       │ HTTPS
┌──────────────────────▼──────────────────────────────────┐
│              Rust Backend REST API                      │
│  (agents/backend/crates/api/src/rest.rs)               │
└─────────────────────────────────────────────────────────┘
```

### State Management

**Current Approach**: React Hooks + Custom Hooks

- **Local State**: `useState` for component-specific state
- **Shared State**: Custom hooks (`useSnapshot`, `useBoxSpreadData`)
- **Side Effects**: `useEffect` for data fetching
- **Memoization**: `useMemo`, `useCallback` for performance

**Future Consideration**: Context API or Zustand for global state if needed

## Wireframes

### Layout Structure

```
┌─────────────────────────────────────────────────────────────┐
│  HeaderStatus                                               │
│  [Mode: DRY-RUN] [Account: DU123456] [Strategy: RUNNING]    │
│  [Start] [Stop] [Toggle Mode]                               │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│  Scenario Summary                                           │
│  Total Scenarios: 15 | Avg APR: 12.5% | Best: SPX @ 15.2%  │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│  Box Spread Table                                           │
│  Symbol │ Strike │ Net Debit │ Profit │ ROI │ APR │ Action│
│  SPX    │ 100    │ 95.50     │ 4.50   │ 4.7 │12.5 │ [Buy] │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│  Tab Navigation                                             │
│  [Dashboard] [Current] [Historic] [Orders] [Alerts]       │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│  Tab Content (Dashboard)                                    │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐          │
│  │ Symbol Grid │ │ Watchlist   │ │ Quick Stats │          │
│  │ SPX 509.20  │ │ + Add Symbol│ │ Net Liq     │          │
│  │ XSP 509.18  │ │ SPX ✓       │ │ $100,523    │          │
│  │ NANOS 509.15│ │ XSP ✓       │ │ Buying Power│          │
│  └─────────────┘ └─────────────┘ │ $80,412     │          │
│                                  └─────────────┘          │
└─────────────────────────────────────────────────────────────┘
```

### Dashboard Tab Wireframe

```
┌─────────────────────────────────────────────────────────────┐
│  Dashboard                                                  │
├─────────────────────────────────────────────────────────────┤
│  Symbols (Watchlist: SPX, XSP, NANOS)                       │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Symbol │ Last │ Bid │ Ask │ Spread │ ROI │ Volume │   │
│  │ SPX    │509.20│509.15│509.18│ 0.03  │0.65 │  120  │   │
│  │        │      │      │      │       │     │ [📈]  │   │
│  │ XSP    │509.18│509.16│509.19│ 0.03  │0.62 │   95  │   │
│  │        │      │      │      │       │     │ [📈]  │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  Quick Actions                                              │
│  [B] Buy Combo  [Shift+S] Sell Combo  [K] Keyboard Shortcuts│
└─────────────────────────────────────────────────────────────┘
```

### Positions Tab Wireframe

```
┌─────────────────────────────────────────────────────────────┐
│  Current Positions                                           │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Symbol │ Qty │ Avg Price │ Current │ PnL │ Actions │   │
│  │ SPX    │  1  │  509.50   │ 510.25  │+0.75│ [View]  │   │
│  │ XSP    │  2  │  509.20   │ 509.18  │-0.04│ [View]  │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  Filters: [All] [Strategy] [Manual] [Symbol: SPX ▼]        │
└─────────────────────────────────────────────────────────────┘
```

### Orders Tab Wireframe

```
┌─────────────────────────────────────────────────────────────┐
│  Orders                                                      │
├─────────────────────────────────────────────────────────────┤
│  Timeline View                                               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 10:00:05 │ SPX │ BUY │ 1 │ PENDING │ [Cancel]      │   │
│  │ 10:00:03 │ XSP │ SELL│ 2 │ FILLED  │ 509.18        │   │
│  │ 09:59:45 │ SPX │ BUY │ 1 │ FILLED  │ 509.50        │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  Status Filter: [All] [Pending] [Filled] [Cancelled]        │
└─────────────────────────────────────────────────────────────┘
```

## Routing Strategy

### Current Implementation

- **No routing library**: Tab-based navigation using state
- **Tab switching**: `useState` for active tab
- **Modal navigation**: State-based modal management

### Future Consideration

- **React Router**: If multi-page navigation needed
- **URL state**: Sync tab selection with URL hash

## State Management Patterns

### Custom Hooks

#### useSnapshot

```typescript
function useSnapshot(apiBaseUrl: string) {
  const [snapshot, setSnapshot] = useState<SnapshotPayload | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    // Polling logic
  }, [apiBaseUrl]);

  return { snapshot, isLoading, error };
}
```

#### useBoxSpreadData

```typescript
function useBoxSpreadData() {
  const [scenarios, setScenarios] = useState<BoxSpreadScenario[]>([]);
  // Fetch box spread scenarios
  return { scenarios, isLoading, error };
}
```

#### useSymbolWatchlist

```typescript
function useSymbolWatchlist() {
  const [watchlist, setWatchlist] = useState<string[]>(['SPX', 'XSP']);
  // Manage symbol watchlist
  return { watchlist, addSymbol, removeSymbol, isDefaultSymbol };
}
```

## API Integration

### Current Endpoints Used

| Endpoint | Method | Hook | Purpose |
|----------|--------|------|---------|
| `/api/v1/snapshot` | GET | `useSnapshot` | Main data source |
| `/api/v1/strategy/start` | POST | `App.tsx` | Start strategy |
| `/api/v1/strategy/stop` | POST | `App.tsx` | Stop strategy |
| `/api/v1/orders/cancel` | POST | `OrdersPanel` | Cancel order |
| `/api/mode` | POST | `HeaderStatus` | Toggle mode |
| `/api/account` | POST | `HeaderStatus` | Change account |
| `/api/v1/scenarios` | GET | `useBoxSpreadData` | Box spread scenarios |

### API Client Pattern

```typescript
// src/api/snapshot.ts
export async function fetchSnapshot(apiBaseUrl: string): Promise<SnapshotPayload> {
  const response = await fetch(`${apiBaseUrl}/api/v1/snapshot`);
  if (!response.ok) throw new Error('Failed to fetch snapshot');
  return response.json();
}
```

## Styling Architecture

### Current Approach

- **Global CSS**: `src/styles/app.css`
- **Dark theme**: Trading desk palette
- **Responsive**: Mobile-first design
- **Component-scoped**: CSS classes per component

### Color Palette

```css
:root {
  --bg-primary: #1a1a1a;
  --bg-secondary: #2a2a2a;
  --text-primary: #ffffff;
  --text-secondary: #cccccc;
  --accent-green: #00ff00;
  --accent-red: #ff0000;
  --accent-yellow: #ffff00;
}
```

## Performance Optimization

### Current Optimizations

- **Memoization**: `useMemo` for expensive calculations
- **Callback memoization**: `useCallback` for event handlers
- **Lazy loading**: Components loaded on demand
- **PWA caching**: Service worker for offline access

### Future Optimizations

- **Code splitting**: Route-based code splitting
- **Virtual scrolling**: For large lists
- **Image optimization**: Lazy loading, WebP format
- **Bundle optimization**: Tree shaking, minification

## PWA Features

### Current Implementation

- ✅ Service worker
- ✅ Offline support
- ✅ Installable
- ✅ Auto updates
- ✅ Smart caching

### Caching Strategy

- **Images**: Cache-first (30 days)
- **API calls**: Network-first (5-minute cache)
- **Data files**: Stale-while-revalidate (1 day)
- **Static assets**: Cache-first

## Accessibility

### Current Support

- Semantic HTML
- ARIA labels (partial)
- Keyboard navigation (partial)

### Future Enhancements

- Full keyboard navigation
- Screen reader support
- High contrast mode
- Reduced motion support

## Testing Strategy

### Current Tests

- **Unit tests**: `App.test.tsx`
- **Testing Library**: React Testing Library
- **Test runner**: Vitest

### Test Coverage

- Component rendering
- User interactions
- API integration (mocked)

## Future Enhancements

### Planned Features

1. **WebSocket Support** (T-15)
   - Real-time updates
   - Push notifications
   - Live order status

2. **Enhanced Keyboard Navigation** (T-13)
   - Full keyboard shortcuts
   - Focus management
   - Accessibility improvements

3. **Advanced Filtering**
   - Multi-criteria filters
   - Saved filter presets
   - Export filtered data

4. **Chart Enhancements**
   - Multiple timeframes
   - Technical indicators
   - Drawing tools

## Wireframe Mockups

### Mobile View (Responsive)

```
┌─────────────────┐
│ HeaderStatus    │
│ [Compact]       │
├─────────────────┤
│ Scenario Summary│
│ [Compact]       │
├─────────────────┤
│ Box Spread Table│
│ [Scrollable]    │
├─────────────────┤
│ Tabs [Mobile]   │
│ [Dashboard]     │
│ [Positions]     │
│ [Orders]        │
└─────────────────┘
```

### Desktop View (Wide)

```
┌─────────────────────────────────────────────────────────┐
│ HeaderStatus (Full Width)                              │
├──────────────┬──────────────────────────────────────────┤
│ Sidebar      │ Main Content                             │
│ - Watchlist  │ - Symbol Grid                            │
│ - Quick Stats│ - Positions Table                       │
│ - Filters    │ - Orders Timeline                       │
└──────────────┴──────────────────────────────────────────┘
```

## Success Criteria

- [ ] Architecture documented
- [ ] Wireframes created
- [ ] Component structure clear
- [ ] Data flow documented
- [ ] API integration complete
- [ ] Performance optimized
- [ ] Accessibility standards met
- [ ] PWA features working
- [ ] Tests comprehensive

## Related Documentation

- [REST API Layer Design](research/architecture/REST_API_LAYER_DESIGN.md)
- [Shared API Contract](../agents/shared/API_CONTRACT.md)
- [iPad Frontend Architecture](IPAD_FRONTEND_ARCHITECTURE.md)
- [Feature Tracking](FEATURE_TRACKING.md)
