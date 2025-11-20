# TUI Test Coverage Analysis

## Current Test Coverage

### ✅ Well Covered

1. **Basic Formatting Functions** (`app_test.go`):
   - `healthColor()` - Health status coloring
   - `severityColor()` - Severity-based coloring
   - `drawCandle()` - Candle visualization

2. **Integration Tests** (`app_integration_test.go`):
   - Help modal display and quit flow

3. **Snapshot Tests** (`app_snapshot_test.go`):
   - Dashboard view rendering
   - Positions tab rendering

4. **REST Provider** (`rest_test.go`):
   - Basic REST provider functionality

## Missing Test Coverage

### 🔴 Critical Gaps (High Priority)

#### 1. UI State Management (`uiState` methods)
**Location**: `app.go` lines 195-326

**Missing Tests**:
- `newUIState()` - State initialization
- `observeSnapshot()` - Snapshot observation and cache updates
- `hasSymbol()` - Symbol existence checking
- `addSymbol()` - Symbol addition with validation
- `currentOrder()` - Order retrieval (natural vs multiplier sort)
- `indexOfSymbol()` - Symbol indexing
- `symbolData()` - Symbol data retrieval (with case-insensitive lookup)
- `setWindowSize()` - Window size changes
- `sparklineWidth()` - Responsive width calculation
- `optionRows()` - Option rows calculation based on window height
- `toggleSortByMultiplier()` - Sort mode toggling
- `curveChartWidth()` - Curve chart width calculation

**Why Critical**: These functions manage core UI state and are used throughout the application. Bugs here would affect all UI interactions.

**Recommended Tests**:
```go
func TestUIState_ObserveSnapshot(t *testing.T)
func TestUIState_AddSymbol(t *testing.T)
func TestUIState_HasSymbol(t *testing.T)
func TestUIState_CurrentOrder(t *testing.T)
func TestUIState_SymbolData(t *testing.T)
func TestUIState_SetWindowSize(t *testing.T)
func TestUIState_SparklineWidth(t *testing.T)
func TestUIState_OptionRows(t *testing.T)
func TestUIState_ToggleSortByMultiplier(t *testing.T)
func TestUIState_CurveChartWidth(t *testing.T)
```

#### 2. Box Spread Calculations
**Location**: `app.go` lines 94-188

**Missing Tests**:
- `selectBenchmark()` - Benchmark selection based on days to expiry
- `computeBoxMetrics()` - Box spread metrics calculation (net debit, profit, APR, etc.)

**Why Critical**: These calculations are core to the trading strategy and must be accurate.

**Recommended Tests**:
```go
func TestSelectBenchmark(t *testing.T)
func TestComputeBoxMetrics(t *testing.T)
```

#### 3. Formatting Functions
**Location**: `app.go` lines 2210-2314

**Missing Tests**:
- `formatBackendLabel()` - Backend label formatting
- `formatAPRSpread()` - APR spread formatting with color
- `drawAPRBar()` - APR bar visualization
- `drawCandle()` - Additional edge cases (already has basic test)

**Why Critical**: Formatting bugs can mislead users about critical trading metrics.

**Recommended Tests**:
```go
func TestFormatBackendLabel(t *testing.T)
func TestFormatAPRSpread(t *testing.T)
func TestDrawAPRBar(t *testing.T)
func TestDrawCandle_EdgeCases(t *testing.T)
```

### 🟡 Important Gaps (Medium Priority)

#### 4. Update Functions
**Location**: `app.go` lines 980-1526

**Missing Tests**:
- `updateHeader()` - Header updates with various snapshot states
- `updateDashboard()` - Dashboard table updates (empty, sorted, unsorted)
- `updateDashboardPositions()` - Position preview updates
- `updatePositions()` - Full positions table updates
- `updateOrders()` - Orders list updates
- `updateAlerts()` - Alerts text view updates
- `updateHistory()` - History table updates
- `updateYieldCurve()` - Yield curve table updates
- `updateFAQ()` - FAQ text view updates
- `updateControls()` - Controls text view updates

**Why Important**: These functions handle all UI updates. While integration tests cover some behavior, unit tests would catch edge cases and formatting issues.

**Recommended Tests**:
```go
func TestUpdateHeader(t *testing.T)
func TestUpdateDashboard_Empty(t *testing.T)
func TestUpdateDashboard_Sorted(t *testing.T)
func TestUpdatePositions(t *testing.T)
func TestUpdateOrders(t *testing.T)
func TestUpdateAlerts(t *testing.T)
func TestUpdateHistory(t *testing.T)
func TestUpdateYieldCurve(t *testing.T)
func TestUpdateFAQ(t *testing.T)
func TestUpdateControls(t *testing.T)
```

#### 5. Modal and Interaction Functions
**Location**: `app.go` lines 1528-2188

**Missing Tests**:
- `promptAddSymbol()` - Symbol addition prompt (validation, duplicate handling)
- `showHelpModal()` - Help modal display
- `showSymbolDetail()` - Symbol detail modal
- `showPositionDetail()` - Position detail modal
- `showHistoricalDetail()` - Historical detail modal
- `showOptionChain()` - Option chain display (complex function with many edge cases)

**Why Important**: These handle user interactions. While integration tests cover basic flows, unit tests would catch validation and edge case bugs.

**Note**: Some of these are difficult to unit test without full tview setup, but key validation logic can be extracted and tested.

#### 6. Data Provider Tests
**Location**: `data/mock.go`, `data/nautilus_placeholder.go`

**Missing Tests**:
- `MockProvider.AddSymbol()` - Symbol addition
- `MockProvider.generateSnapshot()` - Snapshot generation
- `MockProvider` edge cases (empty symbols, invalid data)
- `NautilusPlaceholderProvider` - Basic functionality

**Why Important**: Data providers are critical for testing and development. Bugs here affect all tests.

**Recommended Tests**:
```go
func TestMockProvider_AddSymbol(t *testing.T)
func TestMockProvider_GenerateSnapshot(t *testing.T)
func TestMockProvider_EmptySymbols(t *testing.T)
func TestNautilusPlaceholderProvider(t *testing.T)
```

### 🟢 Nice to Have (Low Priority)

#### 7. Builder Functions
**Location**: `app.go` lines 785-889

**Missing Tests**:
- `buildHeader()`, `buildTabs()`, `buildDashboard()`, etc.

**Why Low Priority**: These are simple builders that create tview primitives. Integration tests already verify they work correctly.

#### 8. Extraction Functions
**Location**: `app.go` lines 891-968

**Missing Tests**:
- `extractDashboardTable()`, `extractDashboardPositions()`, etc.

**Why Low Priority**: These extract components from layouts. Integration tests verify correct extraction.

## Recommendations

### Immediate Actions (High Priority)

1. **Add UI State Tests** (`tui/internal/app/app_state_test.go`):
   - Test all `uiState` methods
   - Focus on edge cases (empty state, invalid inputs, boundary conditions)
   - Test state transitions

2. **Add Calculation Tests** (`tui/internal/app/app_calculations_test.go`):
   - Test `selectBenchmark()` with various DTE values
   - Test `computeBoxMetrics()` with various option chain configurations
   - Test edge cases (zero width, invalid strikes, etc.)

3. **Add Formatting Tests** (`tui/internal/app/app_formatting_test.go`):
   - Test all formatting functions
   - Test edge cases (zero values, negative values, very large values)
   - Test color codes and ANSI sequences

### Short-term Actions (Medium Priority)

4. **Add Update Function Tests** (`tui/internal/app/app_updates_test.go`):
   - Test update functions with various data states
   - Test empty data, large data, edge cases
   - Verify table/list/text view updates

5. **Add Data Provider Tests** (`tui/internal/data/mock_test.go`):
   - Test `MockProvider` thoroughly
   - Test `NautilusPlaceholderProvider`
   - Test error cases and edge conditions

### Long-term Enhancements (Low Priority)

6. **Add Integration Tests for Complex Flows**:
   - Tab navigation (all tabs)
   - Option chain interaction (navigation, box spread calculation)
   - Symbol addition workflow
   - Window resizing behavior

7. **Add Snapshot Tests for More Views**:
   - Yield curve tab
   - History tab
   - Orders tab
   - Alerts tab
   - FAQ tab

8. **Add Performance Tests**:
   - Large dataset rendering
   - Rapid snapshot updates
   - Window resize performance

## Test Organization

### Recommended File Structure

```
tui/internal/app/
├── app_test.go              # Existing: basic formatting
├── app_state_test.go        # NEW: UI state management
├── app_calculations_test.go # NEW: box spread calculations
├── app_formatting_test.go   # NEW: formatting functions
├── app_updates_test.go      # NEW: update functions
├── app_integration_test.go  # Existing: integration tests
└── app_snapshot_test.go    # Existing: snapshot tests

tui/internal/data/
├── rest_test.go             # Existing: REST provider
├── mock_test.go             # NEW: Mock provider tests
└── nautilus_test.go         # NEW: Nautilus provider tests
```

## Testing Patterns

### Given-When-Then Pattern

Follow the pattern established in C++ tests:

```go
func TestUIState_AddSymbol(t *testing.T) {
  // Given: A UI state with existing symbols
  state := newUIState()
  state.addSymbol("SPY")

  // When: Adding a new symbol
  state.addSymbol("QQQ")

  // Then: Symbol should be in watchlist
  if !state.hasSymbol("QQQ") {
    t.Error("QQQ should be in watchlist")
  }
}
```

### Table-Driven Tests

Use table-driven tests for multiple cases:

```go
func TestSelectBenchmark(t *testing.T) {
  tests := []struct {
    name     string
    days     float64
    expected string
  }{
    {"short term", 14, "4-week"},
    {"medium term", 60, "8-week"},
    {"long term", 200, "26-week"},
  }
  for _, tt := range tests {
    t.Run(tt.name, func(t *testing.T) {
      result := selectBenchmark(tt.days)
      if result.label != tt.expected {
        t.Errorf("expected %s, got %s", tt.expected, result.label)
      }
    })
  }
}
```

## Coverage Goals

- **Current**: ~15% (basic formatting + integration)
- **Target**: ~70% (all critical and important functions)
- **Stretch**: ~85% (including edge cases and complex flows)

## Notes

- Some functions (like `showOptionChain()`) are complex and tightly coupled to tview. Consider extracting business logic into testable helper functions.
- Integration tests are valuable but slow. Focus on unit tests for fast feedback.
- Snapshot tests are great for visual regression but require maintenance. Use judiciously.
