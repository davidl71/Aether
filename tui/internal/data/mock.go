package data

import (
	"context"
	"math"
	"strings"
	"sync"
	"time"
)

// MockProvider generates synthetic snapshots for local testing.
type MockProvider struct {
	ch        chan Snapshot
	stopCh    chan struct{}
	once      sync.Once
	mu        sync.Mutex
	symbols   []string
	rngMu     sync.Mutex
	randState uint64
}

// NewMockProvider creates a mock data provider.
func NewMockProvider() *MockProvider {
	seed := uint64(time.Now().UnixNano())
	if seed == 0 {
		seed = 1
	}
	return &MockProvider{
		ch:        make(chan Snapshot, 1),
		stopCh:    make(chan struct{}),
		symbols:   []string{"SPX", "ES50", "NANOS", "XSP"},
		randState: seed,
	}
}

// Start begins emitting snapshots at the specified interval.
func (m *MockProvider) Start(ctx context.Context, interval time.Duration) {
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
				snap := m.generateSnapshot(t)
				select {
				case m.ch <- snap:
				default:
					<-m.ch
					m.ch <- snap
				}
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

// AddSymbol adds a symbol to the mock provider's rotation.
func (m *MockProvider) AddSymbol(symbol string) error {
	cleaned := strings.ToUpper(strings.TrimSpace(symbol))
	if cleaned == "" {
		return nil
	}

	m.mu.Lock()
	defer m.mu.Unlock()
	for _, existing := range m.symbols {
		if existing == cleaned {
			return nil
		}
	}
	m.symbols = append(m.symbols, cleaned)
	return nil
}

func (m *MockProvider) generateSnapshot(now time.Time) Snapshot {
	symbols := m.currentSymbols()

	symbolSnapshots := make([]SymbolSnapshot, 0, len(symbols))
	for _, sym := range symbols {
		base := m.basePriceForSymbol(sym)
		symbolSnapshots = append(symbolSnapshots, m.mockSymbol(sym, base, now))
	}

	positions := []Position{
		m.mockPosition("SPY BOX 675/670", 1, now),
		m.mockPosition("QQQ BOX 390/385", 2, now),
	}

	historic := []Position{
		m.mockHistoric("SPY BOX 670/665", now.Add(-2*time.Hour)),
		m.mockHistoric("QQQ BOX 395/390", now.Add(-4*time.Hour)),
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
		Metrics: AccountMetrics{
			NetLiq:            100500 + m.randFloat()*500,
			BuyingPower:       80400 + m.randFloat()*400,
			ExcessLiquidity:   25000 + m.randFloat()*1000,
			MarginRequirement: 15000 + m.randFloat()*500,
			Commissions:       123.45 + m.randFloat()*5,
			PortalOK:          true,
			TWSOK:             true,
			ORATSOK:           true,
			QuestDBOK:         true,
		},
		Symbols:    symbolSnapshots,
		Positions:  positions,
		Historic:   historic,
		Orders:     orders,
		Alerts:     alerts,
		History:    m.mockHistory(now),
		YieldCurve: m.mockYieldCurve(now),
		FAQs:       mockFAQs(),
	}
}

func (m *MockProvider) currentSymbols() []string {
	m.mu.Lock()
	defer m.mu.Unlock()
	out := make([]string, len(m.symbols))
	copy(out, m.symbols)
	return out
}

func (m *MockProvider) basePriceForSymbol(symbol string) float64 {
	switch symbol {
	case "SPY":
		return 509.0
	case "QQQ":
		return 389.0
	case "IWM":
		return 201.0
	default:
		return 100.0 + m.randFloat()*50.0
	}
}

func (m *MockProvider) randFloat() float64 {
	m.rngMu.Lock()
	m.randState = m.randState*6364136223846793005 + 1
	state := m.randState
	m.rngMu.Unlock()
	return float64((state>>11)&((1<<53)-1)) / (1 << 53)
}

func (m *MockProvider) mockSymbol(symbol string, base float64, now time.Time) SymbolSnapshot {
	last := base + m.randFloat()
	bid := last - 0.03
	ask := last + 0.03
	spread := ask - bid
	roi := (m.randFloat()*0.8 + 0.2) * 100
	maker := int(m.randFloat() * 3)
	taker := int(m.randFloat() * 2)
	volume := 80 + int(m.randFloat()*50)
	candle := m.mockCandle(last, base, now)

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
		OptionChains: m.mockOptionChains(last, now),
	}
}

func (m *MockProvider) mockPosition(name string, qty int, now time.Time) Position {
	roi := (m.randFloat()*0.6 + 0.2) * 100
	maker := int(m.randFloat() * 3)
	taker := int(m.randFloat() * 2)
	rebate := m.randFloat() * 2
	vega := m.randFloat() * 0.5
	theta := (m.randFloat()*0.2 - 0.1)
	fairDiff := (m.randFloat()*0.2 - 0.1) * 5
	candle := m.mockCandle(160+m.randFloat()*5, 160, now)
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

func (m *MockProvider) mockHistoric(name string, timestamp time.Time) Position {
	p := m.mockPosition(name, 0, timestamp)
	p.Quantity = 0
	return p
}

func (m *MockProvider) mockCandle(current, base float64, now time.Time) Candle {
	high := math.Max(current+m.randFloat()*0.5, current)
	low := math.Min(current-m.randFloat()*0.5, current)
	open := base + m.randFloat()*0.5
	volume := 50 + m.randFloat()*20
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

func (m *MockProvider) mockOptionChains(last float64, now time.Time) []OptionSeries {
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
		callMid := intrinsicCall + m.randFloat()*1.5 + 0.2
		putMid := intrinsicPut + m.randFloat()*1.5 + 0.2
		callSpread := m.randFloat()*0.15 + 0.05
		putSpread := m.randFloat()*0.15 + 0.05
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

func (m *MockProvider) mockHistory(now time.Time) []HistoryEntry {
	records := make([]HistoryEntry, 0, 12)
	expiries := []int{7, 14, 28, 56, 91, 182}
	for i, days := range expiries {
		date := now.AddDate(0, 0, -(i+1)*3)
		width := 5.0
		if i%2 == 0 {
			width = 10.0
		}
		netDebit := 100.0 + float64(i)*12.5 + m.randFloat()*2
		apr := 4.5 + float64(i)*0.35 + m.randFloat()*0.25
		benchmark := 4.8 + float64(i)*0.1
		records = append(records, HistoryEntry{
			Date:          date,
			Symbol:        "SPX",
			Expiration:    now.AddDate(0, 0, days).Format("2006-01-02"),
			Width:         width,
			NetDebit:      netDebit,
			APR:           apr,
			Benchmark:     "BIL",
			BenchmarkRate: benchmark,
			Notes:         "Synthetic funding snapshot",
			DaysToExpiry:  float64(days),
		})
	}
	return records
}

func (m *MockProvider) mockYieldCurve(now time.Time) []YieldCurvePoint {
	points := make([]YieldCurvePoint, 0, 8)
	tenors := []struct {
		label string
		days  int
	}{
		{"12D", 12},
		{"1M", 30},
		{"2M", 60},
		{"3M", 90},
		{"6M", 180},
		{"9M", 270},
		{"1Y", 360},
		{"18M", 540},
	}
	for idx, tenor := range tenors {
		baseAPR := 5.0 + float64(idx)*0.1
		apr := baseAPR + (m.randFloat()-0.5)*0.4
		benchmark := 4.8 + float64(idx)*0.08
		netDebit := 90.0 + float64(idx)*15 + m.randFloat()*3
		points = append(points, YieldCurvePoint{
			Label:      tenor.label,
			Expiration: now.AddDate(0, 0, tenor.days).Format("2006-01-02"),
			DTE:        float64(tenor.days),
			NetDebit:   netDebit,
			APR:        apr,
			Benchmark:  benchmark,
			APRSpread:  apr - benchmark,
		})
	}
	return points
}

func mockFAQs() []FAQEntry {
	return []FAQEntry{
		{
			Question: "What is a box spread?",
			Answer:   "A four-leg options strategy combining a bull call spread and bear put spread with matching strikes to synthetically borrow or lend cash.",
		},
		{
			Question: "How is the APR calculated?",
			Answer:   "APR is annualized from the net profit percentage using 365-day basis: APR = profit_pct * (365 / days_to_expiry).",
		},
		{
			Question: "Why compare against T-bill benchmarks?",
			Answer:   "Treasury bills provide the prevailing risk-free funding rate. Comparing APR to a nearby tenor highlights funding edge or drag.",
		},
	}
}
