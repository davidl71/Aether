package app

import (
	"context"
	"strings"
	"testing"
	"time"

	"github.com/davidlowes/ib_box_spread_full_universal/tui/internal/data"
	"github.com/gdamore/tcell/v2"
)

type stubProvider struct {
	ch chan data.Snapshot
}

func newStubProvider(snapshot data.Snapshot) *stubProvider {
	ch := make(chan data.Snapshot, 1)
	ch <- snapshot
	close(ch)
	return &stubProvider{ch: ch}
}

func (p *stubProvider) Snapshots() <-chan data.Snapshot {
	return p.ch
}

func (p *stubProvider) Stop() {}

func TestTUIHelpAndQuit(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping interactive TUI test in short mode")
	}

	screen := tcell.NewSimulationScreen("UTF-8")
	if err := screen.Init(); err != nil {
		t.Fatalf("init simulation screen: %v", err)
	}
	screen.SetSize(120, 40)

	now := time.Now()
	snapshot := data.Snapshot{
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
				OptionChains: []data.OptionSeries{
					{
						Expiration: now.Add(30 * 24 * time.Hour).Format("2006-01-02"),
						Strikes: []data.OptionStrike{
							{Strike: 505, CallBid: 6.10, CallAsk: 6.40, PutBid: 1.25, PutAsk: 1.40},
							{Strike: 510, CallBid: 3.10, CallAsk: 3.30, PutBid: 3.00, PutAsk: 3.20},
							{Strike: 515, CallBid: 1.20, CallAsk: 1.35, PutBid: 6.10, PutAsk: 6.45},
						},
					},
				},
			},
		},
		Orders: []data.Order{{
			Timestamp: now.Add(-time.Minute),
			Text:      "SPY BOX submitted",
			Severity:  "info",
		}},
		Alerts: []data.Alert{{
			Timestamp: now.Add(-30 * time.Second),
			Text:      "All systems nominal",
			Severity:  "success",
		}},
	}

	provider := newStubProvider(snapshot)

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	errCh := make(chan error, 1)
	go func() {
		errCh <- runWithOptions(ctx, runOptions{screen: screen, provider: provider})
	}()

	waitFor := func(label, needle string, timeout time.Duration) {
		deadline := time.Now().Add(timeout)
		for time.Now().Before(deadline) {
			if screenContains(screen, needle) {
				return
			}
			time.Sleep(50 * time.Millisecond)
		}
		t.Fatalf("%s: expected to find %q in screen\n%s", label, needle, screenDump(screen, 30))
	}

	waitAbsent := func(label, needle string, timeout time.Duration) {
		deadline := time.Now().Add(timeout)
		for time.Now().Before(deadline) {
			if !screenContains(screen, needle) {
				return
			}
			time.Sleep(50 * time.Millisecond)
		}
		t.Fatalf("%s: expected %q to disappear\n%s", label, needle, screenDump(screen, 30))
	}

	waitFor("dashboard", "IB Box Spread Terminal", 2*time.Second)

	screen.InjectKey(tcell.KeyRune, '?', tcell.ModNone)
	waitFor("help", "Keyboard Shortcuts", 2*time.Second)

	screen.InjectKey(tcell.KeyRune, 'q', tcell.ModNone)
	waitAbsent("help dismissed", "Keyboard Shortcuts", 2*time.Second)
	waitFor("dashboard redraw", "Dashboard", 2*time.Second)

	screen.InjectKey(tcell.KeyRune, 'q', tcell.ModNone)

	select {
	case err := <-errCh:
		if err != nil && err != context.Canceled {
			t.Fatalf("tui exited with error: %v\n%s", err, screenDump(screen, 30))
		}
	case <-time.After(2 * time.Second):
		t.Fatalf("timed out waiting for tui shutdown\n%s", screenDump(screen, 30))
	}
}

func screenContains(screen tcell.SimulationScreen, needle string) bool {
	return strings.Contains(screenDump(screen, 0), needle)
}

func screenDump(screen tcell.SimulationScreen, rows int) string {
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
