package data

import (
	"context"
	"math"
	"math/rand"
	"sync"
	"time"
)

// MockProvider generates synthetic snapshots for local testing.
type MockProvider struct {
	ch     chan Snapshot
	stopCh chan struct{}
	once   sync.Once
}

// NewMockProvider creates a mock data provider.
func NewMockProvider() *MockProvider {
	return &MockProvider{
		ch:     make(chan Snapshot, 1),
		stopCh: make(chan struct{}),
	}
}

// Start begins emitting snapshots at the specified interval.
func (m *MockProvider) Start(ctx context.Context, interval time.Duration) {
	rand.Seed(time.Now().UnixNano())
	ticker := time.NewTicker(interval)
	go func() {
		defer ticker.Stop()
		for {
			select {
			case <-ctx.Done():
				m.Stop()
				return
			case <-m.stopCh:
				close(m.ch)
				return
			case t := <-ticker.C:
				m.ch <- generateSnapshot(t)
			}
		}
	}()
}

// Snapshots returns a channel of snapshots.
func (m *MockProvider) Snapshots() <-chan Snapshot {
	return m.ch
}

// Stop ends snapshot emission.
func (m *MockProvider) Stop() {
	m.once.Do(func() {
		close(m.stopCh)
	})
}

func generateSnapshot(now time.Time) Snapshot {
	metrics := AccountMetrics{
		NetLiq:            100500 + rand.Float64()*500,
		BuyingPower:       80400 + rand.Float64()*400,
		ExcessLiquidity:   25000 + rand.Float64()*1000,
		MarginRequirement: 15000 + rand.Float64()*500,
		Commissions:       123.45 + rand.Float64()*5,
		PortalOK:          true,
		TWSOK:             true,
		ORATSOK:           true,
		QuestDBOK:         true,
	}

	symbols := []SymbolSnapshot{
		mockSymbol("SPY", 509.0, now),
		mockSymbol("QQQ", 389.0, now),
		mockSymbol("IWM", 201.0, now),
	}

	positions := []Position{
		mockPosition("SPY BOX 675/670", 1, now),
		mockPosition("QQQ BOX 390/385", 2, now),
	}

	historic := []Position{
		mockHistoric("SPY BOX 670/665", now.Add(-2*time.Hour)),
		mockHistoric("QQQ BOX 395/390", now.Add(-4*time.Hour)),
	}

	orders := []Order{
		{Timestamp: now.Add(-time.Minute), Text: "SPY BOX submitted @ 160.00", Severity: "info"},
		{Timestamp: now.Add(-50 * time.Second), Text: "SPY BOX fill 1/4 added liquidity", Severity: "success"},
		{Timestamp: now.Add(-40 * time.Second), Text: "SPY BOX fill 2/4 removed liquidity", Severity: "warn"},
	}

	alerts := []Alert{
		{Timestamp: now.Add(-30 * time.Second), Text: "Combo quote missing for SPY – using ORATS fallback", Severity: "warn"},
		{Timestamp: now.Add(-10 * time.Second), Text: "Portal summary net_liq=100,523 buying_power=80,412", Severity: "info"},
	}

	return Snapshot{
		GeneratedAt: now,
		Mode:        "DRY-RUN",
		Strategy:    "RUNNING",
		AccountID:   "DU123456",
		Metrics:     metrics,
		Symbols:     symbols,
		Positions:   positions,
		Historic:    historic,
		Orders:      orders,
		Alerts:      alerts,
	}
}

func mockSymbol(symbol string, base float64, now time.Time) SymbolSnapshot {
	last := base + rand.NormFloat64()
	bid := last - 0.03
	ask := last + 0.03
	spread := ask - bid
	roi := (rand.Float64()*0.8 + 0.2) * 100
	maker := rand.Intn(3)
	taker := rand.Intn(2)
	volume := 80 + rand.Intn(50)
	candle := mockCandle(last, base, now)

	return SymbolSnapshot{
		Symbol:       symbol,
		Last:         last,
		Bid:          bid,
		Ask:          ask,
		Spread:       spread,
		ROI:          roi,
		MakerCount:   maker,
		TakerCount:   taker,
		Volume:       volume,
		Candle:       candle,
		OptionChains: mockOptionChains(last, now),
	}
}

func mockPosition(name string, qty int, now time.Time) Position {
	roi := (rand.Float64()*0.6 + 0.2) * 100
	maker := rand.Intn(3)
	taker := rand.Intn(2)
	rebate := rand.Float64() * 2
	vega := rand.Float64() * 0.5
	theta := (rand.Float64()*0.2 - 0.1)
	fairDiff := (rand.Float64()*0.2 - 0.1) * 5
	candle := mockCandle(160+rand.Float64()*5, 160, now)
	return Position{
		Name:           name,
		Quantity:       qty,
		ROI:            roi,
		MakerCount:     maker,
		TakerCount:     taker,
		RebateEstimate: rebate,
		Vega:           vega,
		Theta:          theta,
		FairDiff:       fairDiff,
		Candle:         candle,
	}
}

func mockHistoric(name string, timestamp time.Time) Position {
	p := mockPosition(name, 0, timestamp)
	p.Quantity = 0
	return p
}

func mockCandle(current, base float64, now time.Time) Candle {
	high := math.Max(current+rand.Float64()*0.5, current)
	low := math.Min(current-rand.Float64()*0.5, current)
	open := base + rand.Float64()*0.5
	volume := 50 + rand.Float64()*20
	entry := base
	return Candle{
		Open:    open,
		High:    high,
		Low:     low,
		Close:   current,
		Volume:  volume,
		Entry:   entry,
		Updated: now,
	}
}

func mockOptionChains(last float64, now time.Time) []OptionSeries {
	rounded := math.Round(last/5.0) * 5.0
	if rounded <= 0 {
		rounded = last
	}
	strikes := make([]OptionStrike, 0, 11)
	for i := -5; i <= 5; i++ {
		strike := rounded + float64(i)*5
		if strike <= 0 {
			continue
		}
		intrinsicCall := math.Max(last-strike, 0)
		intrinsicPut := math.Max(strike-last, 0)
		callMid := intrinsicCall + rand.Float64()*1.5 + 0.2
		putMid := intrinsicPut + rand.Float64()*1.5 + 0.2
		callSpread := rand.Float64()*0.15 + 0.05
		putSpread := rand.Float64()*0.15 + 0.05
		strikes = append(strikes, OptionStrike{
			Strike:  strike,
			CallBid: math.Max(0, callMid-callSpread/2),
			CallAsk: callMid + callSpread/2,
			PutBid:  math.Max(0, putMid-putSpread/2),
			PutAsk:  putMid + putSpread/2,
		})
	}

	expiry := now.Add(30 * 24 * time.Hour).Format("2006-01-02")
	return []OptionSeries{
		{
			Expiration: expiry,
			Strikes:    strikes,
		},
	}
}
