# TUI Box Spread Scenario Explorer Design

**Date**: 2025-11-17
**Status**: Design Document
**Purpose**: Design specification for adding box spread scenario explorer to TUI

---

## Overview

Add box spread scenario explorer with summary statistics and scenario table to the TUI, matching the functionality available in the web app.

---

## Current State

**Web App Has:**
- `ScenarioSummary.tsx` - Summary component showing total scenarios, average APR, probable count, max APR
- `BoxSpreadTable.tsx` - Full table of all scenarios with sortable columns
- Data from `useBoxSpreadData` hook

**TUI Missing:**
- Scenario explorer view
- Summary statistics display
- Scenario table

---

## Design Specification

### 1. Scenario Summary Section

**Location**: Add to Dashboard tab or create new "Scenarios" tab

**Display:**
```
┌─────────────────────────────────────────────────────────┐
│ Box Spread Scenarios                                    │
├─────────────────────────────────────────────────────────┤
│ Total Scenarios: 42                                     │
│ Average APR: 12.5%                                      │
│ Probable (fill_prob > 0): 18                           │
│ Max APR: 25.3% (SPX 2025-12-19 5000/5100)              │
└─────────────────────────────────────────────────────────┘
```

**Implementation:**
- Use FTXUI `text()` and `hbox()` for layout
- Fetch scenario data from same source as web app
- Calculate statistics in C++ or receive from backend

---

### 2. Scenario Table

**Location**: Below summary or in separate tab

**Columns:**
- Symbol (e.g., "SPX")
- Expiration (e.g., "2025-12-19")
- Strike Width (e.g., "100")
- Net Debit (e.g., "$95.50")
- Profit (e.g., "$4.50")
- ROI % (e.g., "4.7%")
- APR % (e.g., "12.5%")
- Fill Probability (e.g., "0.85")

**Display:**
```
┌──────┬────────────┬──────────────┬───────────┬────────┬───────┬───────┬──────────────┐
│Symbol│Expiration  │Strike Width  │Net Debit  │Profit  │ROI %  │APR %  │Fill Prob    │
├──────┼────────────┼──────────────┼───────────┼────────┼───────┼───────┼──────────────┤
│SPX   │2025-12-19  │100           │$95.50    │$4.50   │4.7%   │12.5%  │0.85         │
│SPX   │2025-12-19  │200           │$190.00   │$10.00  │5.3%   │14.2%  │0.72         │
│...   │...         │...           │...       │...     │...    │...    │...          │
└──────┴────────────┴──────────────┴───────────┴────────┴───────┴───────┴──────────────┘
```

**Implementation:**
- Use FTXUI `Table` component or custom table rendering
- Sortable columns (click header to sort)
- Scrollable list
- Selectable rows (arrow keys to navigate, Enter to view details)

---

## Data Source

**Option 1: Fetch from Backend API**
- Call `/api/v1/scenarios` or similar endpoint
- Parse JSON response
- Cache and refresh periodically

**Option 2: Read from File**
- Backend writes scenario data to JSON file
- TUI reads file (similar to snapshot)
- Poll for updates

**Option 3: Calculate in TUI**
- Use box spread strategy to calculate scenarios
- Display results directly
- Real-time calculation

**Recommendation**: Option 2 (file-based) for consistency with current snapshot approach

---

## Integration Points

**Files to Modify:**
- `native/src/tui_app.cpp` - Add scenario explorer rendering
- `native/src/tui_data.h` - Add scenario data structures
- `native/src/tui_provider.cpp` - Add scenario data fetching

**New Functions:**
- `RenderScenarioExplorer()` - Main rendering function
- `RenderScenarioSummary()` - Summary statistics
- `RenderScenarioTable()` - Scenario table

---

## User Interaction

**Keyboard Navigation:**
- Arrow keys: Navigate table rows
- Enter: View scenario details
- Tab: Switch between summary and table
- Sort: Click column header (if supported)

**Display Options:**
- Filter by symbol
- Filter by expiration
- Sort by APR (descending)
- Show only probable scenarios (fill_prob > 0)

---

## Implementation Steps

1. **Add Data Structures**
   - Define `Scenario` struct in `tui_data.h`
   - Add scenario list to `Snapshot` or separate structure

2. **Add Data Fetching**
   - Extend `Provider` to fetch scenario data
   - Parse JSON scenario data
   - Cache in memory

3. **Add Rendering**
   - Create `RenderScenarioExplorer()` function
   - Add summary section
   - Add table section
   - Integrate into dashboard or new tab

4. **Add Interaction**
   - Keyboard navigation
   - Row selection
   - Detail view

---

## Success Criteria

- [ ] Scenario summary displays correctly
- [ ] Scenario table shows all scenarios
- [ ] Statistics match web app
- [ ] Keyboard navigation works
- [ ] Data refreshes periodically
- [ ] Performance acceptable (< 100ms render time)

---

**Document Status**: ✅ Complete - Design specification ready for implementation
