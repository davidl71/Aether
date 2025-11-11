# TUI Testing Guide

This guide covers automated testing strategies for the Terminal User Interface (TUI) built with Go, tcell, and tview.

## Overview

The TUI uses `tcell.SimulationScreen` for headless testing, allowing automated tests without requiring a real terminal or manual interaction.

## Testing Strategy

### 1. Unit Tests
Test individual functions and components in isolation.

**Location**: `tui/internal/app/app_test.go`

**Examples**:
- Color formatting functions
- Data transformation functions
- Helper utilities

### 2. Integration Tests
Test full TUI flow with simulated screen and keyboard input.

**Location**: `tui/internal/app/app_integration_test.go`

**Features**:
- Simulated screen (120x40)
- Keyboard input injection
- Screen content verification
- Timeout handling

### 3. Snapshot Tests
Capture and compare TUI output for visual regression testing.

**Location**: `tui/internal/app/app_snapshot_test.go` (to be created)

**Purpose**:
- Detect unintended UI changes
- Ensure consistent formatting
- Validate layout changes

## Running Tests

### Run All Tests
```bash
cd tui
go test ./...
```

### Run Specific Test Suite
```bash
# Unit tests only
go test -run TestHealthColor

# Integration tests
go test -run TestTUIHelpAndQuit

# Skip integration tests (fast)
go test -short ./...
```

### Run with Verbose Output
```bash
go test -v ./...
```

### Run with Coverage
```bash
go test -cover ./...
go test -coverprofile=coverage.out ./...
go tool cover -html=coverage.out
```

## Test Automation Scripts

### Quick Test Script
```bash
#!/bin/bash
# scripts/test_tui.sh

set -e

echo "Running TUI tests..."
cd tui

# Run unit tests
echo "Running unit tests..."
go test -short ./...

# Run integration tests (if not short mode)
if [ "$1" != "--short" ]; then
    echo "Running integration tests..."
    go test -run TestTUIHelpAndQuit
fi

# Generate coverage report
echo "Generating coverage report..."
go test -coverprofile=coverage.out ./...
go tool cover -func=coverage.out | tail -1

echo "✅ All tests passed!"
```

### Snapshot Test Script
```bash
#!/bin/bash
# scripts/test_tui_snapshots.sh

set -e

cd tui

# Update snapshots (use with caution)
if [ "$1" == "--update" ]; then
    echo "⚠️  Updating snapshots..."
    UPDATE_SNAPSHOTS=1 go test -run TestTUISnapshots
else
    echo "Running snapshot tests..."
    go test -run TestTUISnapshots
fi
```

## Writing New Tests

### Unit Test Example

```go
func TestFormatROI(t *testing.T) {
    tests := []struct {
        name     string
        roi      float64
        expected string
    }{
        {"positive", 5.5, "[green]5.50%[-]"},
        {"negative", -2.3, "[red]-2.30%[-]"},
        {"zero", 0.0, "0.00%"},
    }

    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            got := formatROI(tt.roi)
            if got != tt.expected {
                t.Errorf("formatROI(%f) = %q, want %q", tt.roi, got, tt.expected)
            }
        })
    }
}
```

### Integration Test Example

```go
func TestTUITabNavigation(t *testing.T) {
    if testing.Short() {
        t.Skip("skipping integration test in short mode")
    }

    screen := tcell.NewSimulationScreen("UTF-8")
    if err := screen.Init(); err != nil {
        t.Fatalf("init screen: %v", err)
    }
    screen.SetSize(120, 40)

    // Setup test data
    snapshot := createTestSnapshot()
    provider := newStubProvider(snapshot)

    ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
    defer cancel()

    errCh := make(chan error, 1)
    go func() {
        errCh <- runWithOptions(ctx, runOptions{
            screen:   screen,
            provider: provider,
        })
    }()

    // Wait for dashboard to appear
    waitForScreen(screen, "Dashboard", 2*time.Second, t)

    // Press Tab to navigate to next tab
    screen.InjectKey(tcell.KeyTab, 0, tcell.ModNone)
    waitForScreen(screen, "Current Positions", 2*time.Second, t)

    // Press Tab again
    screen.InjectKey(tcell.KeyTab, 0, tcell.ModNone)
    waitForScreen(screen, "Historic Positions", 2*time.Second, t)

    // Quit
    screen.InjectKey(tcell.KeyRune, 'q', tcell.ModNone)

    select {
    case err := <-errCh:
        if err != nil && err != context.Canceled {
            t.Fatalf("tui error: %v", err)
        }
    case <-time.After(2 * time.Second):
        t.Fatal("timeout waiting for shutdown")
    }
}
```

### Snapshot Test Example

```go
func TestTUISnapshotDashboard(t *testing.T) {
    screen := tcell.NewSimulationScreen("UTF-8")
    screen.Init()
    screen.SetSize(120, 40)

    snapshot := createTestSnapshot()
    provider := newStubProvider(snapshot)

    ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
    defer cancel()

    go func() {
        runWithOptions(ctx, runOptions{
            screen:   screen,
            provider: provider,
        })
    }()

    // Wait for dashboard to render
    time.Sleep(500 * time.Millisecond)

    // Capture screen
    actual := screenDump(screen, 0)

    // Compare with snapshot
    snapshotFile := "testdata/snapshots/dashboard.txt"
    if *updateSnapshots {
        os.WriteFile(snapshotFile, []byte(actual), 0644)
        return
    }

    expected, err := os.ReadFile(snapshotFile)
    if err != nil {
        t.Fatalf("read snapshot: %v", err)
    }

    if actual != string(expected) {
        t.Errorf("snapshot mismatch:\n%s", diff(string(expected), actual))
    }
}
```

## Test Helpers

### Screen Utilities

```go
// waitForScreen waits for text to appear on screen
func waitForScreen(screen tcell.SimulationScreen, text string, timeout time.Duration, t *testing.T) {
    deadline := time.Now().Add(timeout)
    for time.Now().Before(deadline) {
        if screenContains(screen, text) {
            return
        }
        time.Sleep(50 * time.Millisecond)
    }
    t.Fatalf("timeout: expected %q\n%s", text, screenDump(screen, 30))
}

// screenContains checks if screen contains text
func screenContains(screen tcell.SimulationScreen, needle string) bool {
    return strings.Contains(screenDump(screen, 0), needle)
}

// screenDump captures screen contents as string
func screenDump(screen tcell.SimulationScreen, maxRows int) string {
    cells, w, h := screen.GetContents()
    if maxRows <= 0 || maxRows > h {
        maxRows = h
    }
    var builder strings.Builder
    for y := 0; y < maxRows; y++ {
        offset := y * w
        for x := 0; x < w; x++ {
            runes := cells[offset+x].Runes
            if len(runes) == 0 {
                builder.WriteRune(' ')
                continue
            }
            builder.WriteRune(runes[0])
        }
        builder.WriteRune('\n')
    }
    return builder.String()
}
```

### Test Data Fixtures

```go
// createTestSnapshot creates a test snapshot with all fields populated
func createTestSnapshot() data.Snapshot {
    now := time.Now()
    return data.Snapshot{
        GeneratedAt: now,
        Mode:        "DRY-RUN",
        Strategy:    "RUNNING",
        AccountID:   "DU123456",
        Metrics: data.AccountMetrics{
            NetLiq:            100_000,
            BuyingPower:       80_000,
            ExcessLiquidity:   50_000,
            MarginRequirement: 20_000,
            Commissions:       123.45,
            PortalOK:          true,
            TWSOK:             true,
            ORATSOK:           true,
            QuestDBOK:         true,
        },
        Symbols: []data.SymbolSnapshot{
            {
                Symbol:     "SPY",
                Last:       510.12,
                Bid:        510.05,
                Ask:        510.19,
                Spread:     0.14,
                ROI:        1.42,
                MakerCount: 2,
                TakerCount: 1,
                Volume:     100,
            },
        },
        Orders: []data.Order{
            {
                Timestamp: now.Add(-time.Minute),
                Text:      "SPY BOX submitted",
                Severity:  "info",
            },
        },
        Alerts: []data.Alert{
            {
                Timestamp: now.Add(-30 * time.Second),
                Text:      "All systems nominal",
                Severity:  "success",
            },
        },
    }
}
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: TUI Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Set up Go
        uses: actions/setup-go@v4
        with:
          go-version: '1.21'

      - name: Run unit tests
        run: |
          cd tui
          go test -short ./...

      - name: Run integration tests
        run: |
          cd tui
          go test -run TestTUIHelpAndQuit

      - name: Generate coverage
        run: |
          cd tui
          go test -coverprofile=coverage.out ./...
          go tool cover -html=coverage.out -o coverage.html

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          file: ./tui/coverage.out
```

## Visual Regression Testing

### Snapshot Testing Setup

1. **Create snapshot directory**:
   ```bash
   mkdir -p tui/testdata/snapshots
   ```

2. **Generate initial snapshots**:
   ```bash
   UPDATE_SNAPSHOTS=1 go test -run TestTUISnapshots
   ```

3. **Review snapshots**:
   ```bash
   cat tui/testdata/snapshots/dashboard.txt
   ```

4. **Update snapshots when UI changes**:
   ```bash
   UPDATE_SNAPSHOTS=1 go test -run TestTUISnapshots
   ```

### Snapshot Management

- **Commit snapshots** to git (they're test artifacts)
- **Review diffs** when snapshots change
- **Update snapshots** intentionally when UI changes
- **Don't auto-update** in CI (fail on mismatch)

## Best Practices

### 1. Test Structure
- Use table-driven tests for multiple cases
- Keep tests focused and isolated
- Use descriptive test names

### 2. Test Data
- Create reusable test fixtures
- Use realistic test data
- Test edge cases (empty data, large data, etc.)

### 3. Assertions
- Use clear error messages
- Include context in failures
- Dump screen contents on failure

### 4. Performance
- Use `-short` flag for fast unit tests
- Skip integration tests in short mode
- Use timeouts to prevent hanging tests

### 5. Maintenance
- Update snapshots when UI changes intentionally
- Review snapshot diffs carefully
- Keep test helpers in separate files

## Troubleshooting

### Tests Hang
- Check for missing context cancellation
- Verify timeouts are set
- Ensure goroutines exit properly

### Screen Not Rendering
- Verify screen initialization
- Check screen size is set
- Ensure provider is sending data

### Snapshot Mismatches
- Review diff carefully
- Check if change is intentional
- Update snapshot if change is expected

## Related Documentation

- **TUI Design**: `docs/TUI_DESIGN.md`
- **Go Testing**: https://pkg.go.dev/testing
- **tcell Documentation**: https://pkg.go.dev/github.com/gdamore/tcell/v2
