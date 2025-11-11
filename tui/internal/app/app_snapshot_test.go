package app

import (
	"context"
	"flag"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/davidlowes/ib_box_spread_full_universal/tui/internal/data"
	"github.com/gdamore/tcell/v2"
)

// waitFor waits for text to appear on screen
func waitFor(label, needle string, timeout time.Duration, screen tcell.SimulationScreen, t *testing.T) {
	deadline := time.Now().Add(timeout)
	for time.Now().Before(deadline) {
		if strings.Contains(snapshotScreenDump(screen, 0), needle) {
			return
		}
		time.Sleep(50 * time.Millisecond)
	}
	t.Fatalf("%s: expected to find %q in screen\n%s", label, needle, snapshotScreenDump(screen, 30))
}

// snapshotScreenDump captures screen contents for snapshot tests
func snapshotScreenDump(screen tcell.SimulationScreen, rows int) string {
	cells, w, h := screen.GetContents()
	if rows <= 0 || rows > h {
		rows = h
	}
	var builder strings.Builder
	for y := 0; y < rows; y++ {
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

var updateSnapshots = flag.Bool("update-snapshots", false, "update snapshot files")

func TestTUISnapshotDashboard(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping snapshot test in short mode")
	}

	screen := tcell.NewSimulationScreen("UTF-8")
	if err := screen.Init(); err != nil {
		t.Fatalf("init screen: %v", err)
	}
	screen.SetSize(120, 40)

	snapshot := createTestSnapshot()
	provider := newStubProvider(snapshot)

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	errCh := make(chan error, 1)
	go func() {
		errCh <- runWithOptions(ctx, runOptions{
			screen:   screen,
			provider: provider,
		})
	}()

	// Wait for dashboard to render
	waitFor("dashboard", "IB Box Spread Terminal", 2*time.Second, screen, t)
	time.Sleep(200 * time.Millisecond) // Allow UI to stabilize

	// Capture screen (first 30 rows for dashboard)
	actual := snapshotScreenDump(screen, 30)

	// Compare with snapshot
	snapshotFile := filepath.Join("testdata", "snapshots", "dashboard.txt")
	if *updateSnapshots {
		if err := os.MkdirAll(filepath.Dir(snapshotFile), 0755); err != nil {
			t.Fatalf("create snapshot dir: %v", err)
		}
		if err := os.WriteFile(snapshotFile, []byte(actual), 0644); err != nil {
			t.Fatalf("write snapshot: %v", err)
		}
		t.Logf("Updated snapshot: %s", snapshotFile)
		return
	}

	expected, err := os.ReadFile(snapshotFile)
	if err != nil {
		if os.IsNotExist(err) {
			t.Fatalf("snapshot file not found: %s\nRun with -update-snapshots to create it\nActual output:\n%s", snapshotFile, actual)
		}
		t.Fatalf("read snapshot: %v", err)
	}

	if actual != string(expected) {
		t.Errorf("snapshot mismatch for dashboard\nExpected:\n%s\n\nActual:\n%s\n\nDiff:\n%s",
			string(expected), actual, diff(string(expected), actual))
	}
}

func TestTUISnapshotPositionsTab(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping snapshot test in short mode")
	}

	screen := tcell.NewSimulationScreen("UTF-8")
	if err := screen.Init(); err != nil {
		t.Fatalf("init screen: %v", err)
	}
	screen.SetSize(120, 40)

	snapshot := createTestSnapshot()
	provider := newStubProvider(snapshot)

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	errCh := make(chan error, 1)
	go func() {
		errCh <- runWithOptions(ctx, runOptions{
			screen:   screen,
			provider: provider,
		})
	}()

	// Wait for dashboard
	waitFor("dashboard", "Dashboard", 2*time.Second, screen, t)

	// Navigate to Positions tab
	screen.InjectKey(tcell.KeyTab, 0, tcell.ModNone)
	waitFor("positions tab", "Current Positions", 2*time.Second, screen, t)
	time.Sleep(200 * time.Millisecond)

	// Capture screen
	actual := snapshotScreenDump(screen, 30)

	snapshotFile := filepath.Join("testdata", "snapshots", "positions.txt")
	if *updateSnapshots {
		if err := os.MkdirAll(filepath.Dir(snapshotFile), 0755); err != nil {
			t.Fatalf("create snapshot dir: %v", err)
		}
		if err := os.WriteFile(snapshotFile, []byte(actual), 0644); err != nil {
			t.Fatalf("write snapshot: %v", err)
		}
		t.Logf("Updated snapshot: %s", snapshotFile)
		return
	}

	expected, err := os.ReadFile(snapshotFile)
	if err != nil {
		if os.IsNotExist(err) {
			t.Fatalf("snapshot file not found: %s\nRun with -update-snapshots to create it", snapshotFile)
		}
		t.Fatalf("read snapshot: %v", err)
	}

	if actual != string(expected) {
		t.Errorf("snapshot mismatch for positions tab\nExpected:\n%s\n\nActual:\n%s",
			string(expected), actual)
	}

	// Cleanup
	screen.InjectKey(tcell.KeyRune, 'q', tcell.ModNone)
	select {
	case <-errCh:
	case <-time.After(1 * time.Second):
	}
}

// Helper function for snapshot diffing

func diff(expected, actual string) string {
	expectedLines := strings.Split(expected, "\n")
	actualLines := strings.Split(actual, "\n")

	maxLen := len(expectedLines)
	if len(actualLines) > maxLen {
		maxLen = len(actualLines)
	}

	var diff strings.Builder
	for i := 0; i < maxLen; i++ {
		var exp, act string
		if i < len(expectedLines) {
			exp = expectedLines[i]
		}
		if i < len(actualLines) {
			act = actualLines[i]
		}

		if exp != act {
			diff.WriteString(fmt.Sprintf("Line %d:\n", i+1))
			diff.WriteString(fmt.Sprintf("  Expected: %q\n", exp))
			diff.WriteString(fmt.Sprintf("  Actual:   %q\n", act))
		}
	}

	return diff.String()
}

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
				Candle: data.Candle{
					Open:    509.80,
					High:    510.45,
					Low:     509.50,
					Close:   510.12,
					Volume:  120,
					Entry:   509.70,
					Updated: now,
				},
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
