# Feature Tracking: TUI and Web App

This document tracks feature parity between the Terminal User Interface (TUI) and Web Application to ensure both interfaces stay synchronized.

## Feature Status Legend

- ✅ **Implemented** - Feature is fully implemented
- 🚧 **Partial** - Feature is partially implemented or in progress
- ❌ **Missing** - Feature is not yet implemented
- 🔄 **Different** - Feature exists but works differently (noted in comments)

## Core Features

### 1. Header Status Banner

**Purpose**: Display connection status, account info, and system health

| Feature | TUI | Web App | Notes |
|---------|-----|---------|-------|
| Connection status (TWS) | ✅ | ✅ | Shows TWS connection state |
| Account ID display | ✅ | ✅ | Displays account identifier |
| Mode indicator (DRY-RUN/LIVE) | ✅ | ✅ | Shows trading mode |
| Strategy status (RUNNING/STOPPED) | ✅ | ✅ | Shows strategy state |
| Subsystem status (ORATS, Portal, QuestDB) | ✅ | 🚧 | Web shows basic status |
| Timestamp display | ✅ | ✅ | Shows current time |

**TUI Implementation**: `native/src/tui_app.cpp` - `RenderHeader()`
**Web Implementation**: `web/src/components/HeaderStatus.tsx`

---

### 2. Tab Navigation

**Purpose**: Navigate between different views

| Feature | TUI | Web App | Notes |
|---------|-----|---------|-------|
| Dashboard tab | ✅ | ✅ | Main overview |
| Current Positions tab | ✅ | ✅ | Active positions |
| Historic Positions tab | ✅ | ✅ | Closed positions |
| Orders tab | ✅ | ✅ | Order timeline |
| Alerts tab | ✅ | ✅ | Notification feed |
| Keyboard navigation (Tab/Shift+Tab) | ✅ | ❌ | TUI only |
| Mouse/click navigation | N/A | ✅ | Web only |

**TUI Implementation**: `native/src/tui_app.cpp` - `RenderTabs()`
**Web Implementation**: `web/src/components/TabNavigation.tsx`

---

### 3. Dashboard Tab

**Purpose**: Overview of symbols, metrics, and system status

| Feature | TUI | Web App | Notes |
|---------|-----|---------|-------|
| Symbol table/list | ✅ | ✅ | Shows underlying symbols |
| Symbol details (bid/ask/spread) | ✅ | ✅ | Price information |
| ROI display | ✅ | ✅ | Return on investment |
| Maker/Taker counts | ✅ | 🚧 | Web may show differently |
| Volume display | ✅ | ✅ | Trading volume |
| Sparkline/candle visualization | ✅ | ✅ | Price history visualization |
| Symbol selection/detail view | ✅ | ✅ | Click/select for details |
| Account metrics (net liq, buying power) | ✅ | 🚧 | Web may show in header |

**TUI Implementation**: `native/src/tui_app.cpp` - `RenderDashboard()`
**Web Implementation**: `web/src/components/DashboardTab.tsx`

---

### 4. Current Positions Tab

**Purpose**: Display active/open box spread positions

| Feature | TUI | Web App | Notes |
|---------|-----|---------|-------|
| Position list/table | ✅ | ✅ | Shows all active positions |
| Position name/identifier | ✅ | ✅ | Position identifier |
| Quantity display | ✅ | ✅ | Number of contracts |
| ROI per position | ✅ | ✅ | Return on investment |
| Maker/Taker counts | ✅ | 🚧 | Web may show differently |
| Rebate estimate | ✅ | 🚧 | Web may not show |
| Greeks (Vega, Theta) | ✅ | 🚧 | Web may not show |
| Fair value difference | ✅ | 🚧 | Web may not show |
| Candle/sparkline | ✅ | ✅ | Price history |
| Position detail modal | ✅ | ✅ | Click for full details |

**TUI Implementation**: `native/src/tui_app.cpp` - `RenderPositions()`
**Web Implementation**: `web/src/components/PositionsTable.tsx`

---

### 5. Historic Positions Tab

**Purpose**: Display recently closed positions

| Feature | TUI | Web App | Notes |
|---------|-----|---------|-------|
| Historic position list | ✅ | ✅ | Shows closed positions |
| Same fields as Current Positions | ✅ | ✅ | Similar data structure |
| Historical data source (QuestDB) | ✅ | 🚧 | Web may use different source |

**TUI Implementation**: `native/src/tui_app.cpp` - `RenderPositions()` (historic)
**Web Implementation**: `web/src/components/PositionsTable.tsx` (historic prop)

---

### 6. Orders Tab

**Purpose**: Display order timeline and history

| Feature | TUI | Web App | Notes |
|---------|-----|---------|-------|
| Order list/timeline | ✅ | ✅ | Chronological order display |
| Order timestamp | ✅ | ✅ | When order occurred |
| Order details/text | ✅ | ✅ | Order description |
| Order severity (info/warn/error/success) | ✅ | ✅ | Visual indicators |
| Order filtering | 🚧 | ❌ | TUI may have basic filtering |

**TUI Implementation**: `native/src/tui_app.cpp` - `RenderOrders()`
**Web Implementation**: `web/src/components/OrdersPanel.tsx`

---

### 7. Alerts Tab

**Purpose**: Display system alerts and notifications

| Feature | TUI | Web App | Notes |
|---------|-----|---------|-------|
| Alert list/feed | ✅ | ✅ | Chronological alert display |
| Alert timestamp | ✅ | ✅ | When alert occurred |
| Alert text/message | ✅ | ✅ | Alert description |
| Alert severity (info/warn/error) | ✅ | ✅ | Visual indicators |
| Scrollable feed | ✅ | ✅ | Can scroll through alerts |

**TUI Implementation**: `native/src/tui_app.cpp` - `RenderAlerts()`
**Web Implementation**: `web/src/components/AlertsPanel.tsx`

---

### 8. Box Spread Scenario Explorer

**Purpose**: Display box spread scenarios and opportunities

| Feature | TUI | Web App | Notes |
|---------|-----|---------|-------|
| Scenario summary | 🚧 | ✅ | Web has summary component |
| Scenario table | 🚧 | ✅ | Web has full table |
| Total scenarios count | 🚧 | ✅ | Web shows count |
| Average APR | 🚧 | ✅ | Web calculates average |
| Probable scenarios count | 🚧 | ✅ | Web filters by probability |
| Max APR scenario | 🚧 | ✅ | Web highlights best |
| Scenario details (strike width, APR, etc.) | 🚧 | ✅ | Web shows full details |
| Underlying symbol | 🚧 | ✅ | Web shows underlying |
| As-of timestamp | 🚧 | ✅ | Web shows data timestamp |

**TUI Implementation**: May be in dashboard or separate view
**Web Implementation**: `web/src/components/ScenarioSummary.tsx`, `web/src/components/BoxSpreadTable.tsx`

---

### 9. Quick Actions

**Purpose**: Rapid trading actions via keyboard shortcuts

| Feature | TUI | Web App | Notes |
|---------|-----|---------|-------|
| Buy Combo (B key) | ✅ | ✅ | Both support B key |
| Sell Combo (Shift+S) | ✅ | ✅ | Both support Shift+S |
| Action bar/buttons | ✅ | ✅ | Web has visual buttons |
| Action confirmation | 🚧 | 🚧 | May need implementation |
| Action feedback/modal | ✅ | ✅ | Both show feedback |

**TUI Implementation**: `native/src/tui_app.cpp` - Keyboard handlers
**Web Implementation**: `web/src/components/ActionBar.tsx`, `web/src/App.tsx` - Keyboard handlers

---

### 10. Detail Modals/Popovers

**Purpose**: Show detailed information for selected items

| Feature | TUI | Web App | Notes |
|---------|-----|---------|-------|
| Symbol detail modal | ✅ | ✅ | Click symbol for details |
| Position detail modal | ✅ | ✅ | Click position for details |
| Order detail view | 🚧 | 🚧 | May be in timeline |
| Modal close/dismiss | ✅ | ✅ | Both support closing |

**TUI Implementation**: `native/src/tui_app.cpp` - Detail popovers
**Web Implementation**: `web/src/components/DetailModal.tsx`

---

### 11. Keyboard Shortcuts

**Purpose**: Quick navigation and actions

| Feature | TUI | Web App | Notes |
|---------|-----|---------|-------|
| Tab/Shift+Tab (navigate tabs) | ✅ | ❌ | TUI only |
| Arrow keys (scroll) | ✅ | ✅ | Both support scrolling |
| Enter (open detail) | ✅ | 🚧 | Web may use click |
| B (buy combo) | ✅ | ✅ | Both support |
| Shift+S (sell combo) | ✅ | ✅ | Both support |
| S (start strategy) | ✅ | ❌ | TUI only |
| T (stop strategy) | ✅ | ❌ | TUI only |
| K (cancel orders) | ✅ | ❌ | TUI only |
| D (toggle dry-run) | ✅ | ❌ | TUI only |
| Q (quit) | ✅ | N/A | TUI only |

**TUI Implementation**: `native/src/tui_app.cpp` - Event handlers
**Web Implementation**: `web/src/App.tsx` - Keyboard event handlers

---

### 12. Data Sources & Polling

**Purpose**: How data is fetched and updated

| Feature | TUI | Web App | Notes |
|---------|-----|---------|-------|
| REST API polling | ✅ | ✅ | Both poll `/data/snapshot.json` |
| WebSocket support | 🚧 | 🚧 | Both may support in future |
| Snapshot schema | ✅ | ✅ | Both use same Snapshot schema |
| Box spread data | 🚧 | ✅ | Web polls `/data/box_spread_sample.json` |
| Error handling | ✅ | ✅ | Both handle errors |
| Loading states | ✅ | ✅ | Both show loading |

**TUI Implementation**: `native/src/tui_app.cpp` - Data fetching
**Web Implementation**: `web/src/hooks/useSnapshot.ts`, `web/src/hooks/useBoxSpreadData.ts`

---

### 13. Styling & Theming

**Purpose**: Visual appearance and color scheme

| Feature | TUI | Web App | Notes |
|---------|-----|---------|-------|
| Dark theme | ✅ | ✅ | Both use dark palette |
| Color coding (green/red/cyan) | ✅ | ✅ | Both use similar colors |
| Status indicators | ✅ | ✅ | Both show status colors |
| Responsive design | N/A | ✅ | Web only |
| Sparkline styling | ✅ | ✅ | Both show price history |

**TUI Implementation**: `native/src/tui_app.cpp` - Color scheme
**Web Implementation**: `web/src/styles/app.css`

---

### 14. Footer/Status Bar

**Purpose**: Show keyboard shortcuts and help

| Feature | TUI | Web App | Notes |
|---------|-----|---------|-------|
| Context-sensitive help | ✅ | ❌ | TUI shows keys for current tab |
| Footer display | ✅ | ❌ | TUI has htop-style footer |

**TUI Implementation**: `native/src/tui_app.cpp` - `RenderFooter()`
**Web Implementation**: Not implemented

---

## Feature Gaps to Address

### High Priority (Core Functionality)

1. **Web App Missing**:
   - Strategy control (Start/Stop) - TUI has S/T keys
   - Cancel orders - TUI has K key
   - Toggle dry-run mode - TUI has D key
   - Tab navigation via keyboard - TUI has Tab/Shift+Tab
   - Footer with keyboard shortcuts

2. **TUI Missing**:
   - Box spread scenario explorer (Web has full implementation)
   - Scenario summary statistics

3. **Both Need**:
   - WebSocket support for real-time updates
   - Enhanced error handling and retry logic
   - Order detail views
   - Position filtering and sorting

### Medium Priority (Enhanced Features)

1. **Web App**:
   - Maker/Taker count display (TUI shows this)
   - Rebate estimate display (TUI shows this)
   - Greeks display (Vega, Theta) - TUI shows this
   - Fair value difference - TUI shows this
   - Subsystem status details (ORATS, QuestDB)

2. **TUI**:
   - Better scenario explorer integration
   - Enhanced modal/popover system

### Low Priority (Nice to Have)

1. **Both**:
   - Export functionality
   - Print/save reports
   - Customizable views
   - Theme customization

---

## Data Schema Alignment

Both TUI and Web App use the same `Snapshot` schema from `native/include/tui_data.h`:

```cpp
struct Snapshot {
  std::chrono::system_clock::time_point generated_at;
  std::string mode;  // "DRY-RUN", "LIVE"
  std::string strategy;  // "RUNNING", "STOPPED"
  std::string account_id;
  AccountMetrics metrics;
  std::vector<SymbolSnapshot> symbols;
  std::vector<Position> positions;
  std::vector<Position> historic;
  std::vector<Order> orders;
  std::vector<Alert> alerts;
  // ... additional fields
};
```

**Web App Types**: `web/src/types/snapshot.ts` should match this schema.

---

## Implementation Guidelines

### When Adding a New Feature

1. **Update this document** with the new feature
2. **Implement in both** TUI and Web App (or mark as platform-specific)
3. **Use shared data schema** - don't create separate schemas
4. **Test both interfaces** to ensure consistency
5. **Update README files** for both TUI and Web App

### When Modifying an Existing Feature

1. **Check this document** to see if feature exists in both
2. **Update both implementations** if it's a shared feature
3. **Update this document** if behavior changes
4. **Maintain API compatibility** - don't break the Snapshot schema

### Platform-Specific Features

Some features are inherently platform-specific:

- **TUI**: Keyboard-only navigation, terminal-specific shortcuts
- **Web**: Mouse/touch interaction, responsive design, PWA features

Mark these clearly in the notes column.

---

## Testing Checklist

When testing feature parity:

- [ ] All tabs render correctly in both
- [ ] Data displays match between TUI and Web
- [ ] Keyboard shortcuts work (where applicable)
- [ ] Detail modals show same information
- [ ] Error states are handled consistently
- [ ] Loading states are shown appropriately
- [ ] Color coding is consistent
- [ ] Snapshot schema is compatible

---

## Related Documentation

- [TUI Design](TUI_DESIGN.md) - TUI-specific design decisions
- [Web App README](../web/README.md) - Web app architecture
- [TUI Data Schema](../native/include/tui_data.h) - Shared data structures
- [API Contract](../agents/shared/API_CONTRACT.md) - REST API specification

---

**Last Updated**: 2025-01-27
**Maintainer**: Development Team
**Review Frequency**: Weekly during active development
