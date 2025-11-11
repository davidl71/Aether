package app

import (
	"context"
	"fmt"
	"log"
	"math"
	"os"
	"sort"
	"strconv"
	"strings"
	"time"

	"github.com/davidlowes/ib_box_spread_full_universal/tui/internal/data"

	"github.com/gdamore/tcell/v2"
	"github.com/rivo/tview"
)

type uiState struct {
	latest             data.Snapshot
	symbolCache        map[string]data.SymbolSnapshot
	watchlist          []string
	dashboardOrder     []string
	windowWidth        int
	windowHeight       int
	dashboardTable     *tview.Table
	dashboardPositions *tview.Table
	dashboardCurve     *tview.Table
	dashboardHistory   *tview.Table
	dashboardOrders    *tview.List
	dashboardAlerts    *tview.TextView
	historyTable       *tview.Table
	yieldCurveTable    *tview.Table
	faqView            *tview.TextView
	symbolAdder        symbolAdder
	sortByMultiplier   bool
	backendLabel       string
	helpShown          bool
	helpBannerExpiry   time.Time
	boxWidth           float64
}

type symbolAdder interface {
	AddSymbol(symbol string) error
}

type runOptions struct {
	screen   tcell.Screen
	provider data.Provider
}

func newMockProvider(ctx context.Context) data.Provider {
	mock := data.NewMockProvider()
	mock.Start(ctx, time.Second)
	return mock
}

func newNautilusProvider(ctx context.Context) data.Provider {
	return data.NewNautilusPlaceholderProvider(ctx, time.Second)
}

type tBillBenchmark struct {
	days    int
	label   string
	symbol  string
	cusip   string
	ratePct float64
}

type boxMetrics struct {
	lowerStrike    float64
	upperStrike    float64
	width          float64
	netDebit       float64
	cashFlow       float64
	payoff         float64
	profit         float64
	profitPct      float64
	aprPct         float64
	benchmark      tBillBenchmark
	aprVsBenchmark float64
	daysToExpiry   float64
}

var tBillBenchmarks = []tBillBenchmark{
	{days: 28, label: "4-week (~28d)", symbol: "BIL", cusip: "IBCID826931582", ratePct: 5.20},
	{days: 56, label: "8-week (~56d)", symbol: "OBIL", cusip: "IBCID826931590", ratePct: 5.22},
	{days: 91, label: "13-week (~91d)", symbol: "TBIL", cusip: "IBCID826931608", ratePct: 5.25},
	{days: 182, label: "26-week (~182d)", symbol: "TFLO", cusip: "IBCID826931616", ratePct: 5.15},
	{days: 364, label: "52-week (~364d)", symbol: "US1Y", cusip: "IBCID826931624", ratePct: 4.90},
}

func selectBenchmark(days float64) tBillBenchmark {
	if days >= float64(tBillBenchmarks[len(tBillBenchmarks)-1].days) {
		return tBillBenchmarks[len(tBillBenchmarks)-1]
	}
	best := tBillBenchmarks[0]
	minDiff := math.MaxFloat64
	for _, candidate := range tBillBenchmarks {
		diff := math.Abs(days - float64(candidate.days))
		if diff < minDiff {
			minDiff = diff
			best = candidate
		}
	}
	return best
}

func computeBoxMetrics(state *uiState, sym data.SymbolSnapshot, series data.OptionSeries) *boxMetrics {
	if state == nil || len(series.Strikes) < 2 {
		return nil
	}
	generated := state.latest.GeneratedAt
	if generated.IsZero() {
		return nil
	}
	expiry, err := time.Parse("2006-01-02", series.Expiration)
	if err != nil {
		return nil
	}
	days := expiry.Sub(generated).Hours() / 24.0
	if days <= 0 {
		return nil
	}

	bestIdx := -1
	minDiff := math.MaxFloat64
	for i := 0; i < len(series.Strikes)-1; i++ {
		lower := series.Strikes[i]
		upper := series.Strikes[i+1]
		center := (lower.Strike + upper.Strike) / 2.0
		diff := math.Abs(center - sym.Last)
		if diff < minDiff {
			minDiff = diff
			bestIdx = i
		}
	}
	if bestIdx < 0 {
		return nil
	}

	lower := series.Strikes[bestIdx]
	upper := series.Strikes[bestIdx+1]
	width := upper.Strike - lower.Strike
	if width <= 0 {
		return nil
	}

	callLowAsk := lower.CallAsk
	callHighBid := upper.CallBid
	putHighAsk := upper.PutAsk
	putLowBid := lower.PutBid
	if callLowAsk <= 0 || putHighAsk <= 0 {
		return nil
	}

	netDebit := callLowAsk - callHighBid + putHighAsk - putLowBid
	multiplier := sym.Multiplier
	if multiplier <= 0 {
		multiplier = 100
	}

	cashFlow := netDebit * multiplier
	if cashFlow <= 0 {
		return nil
	}
	payoff := width * multiplier
	profit := payoff - cashFlow
	profitPct := (profit / cashFlow) * 100.0
	apr := profitPct * (365.0 / days)
	benchmark := selectBenchmark(days)

	return &boxMetrics{
		lowerStrike:    lower.Strike,
		upperStrike:    upper.Strike,
		width:          width,
		netDebit:       netDebit,
		cashFlow:       cashFlow,
		payoff:         payoff,
		profit:         profit,
		profitPct:      profitPct,
		aprPct:         apr,
		benchmark:      benchmark,
		aprVsBenchmark: apr - benchmark.ratePct,
		daysToExpiry:   days,
	}
}

type tabDefinition struct {
	id    string
	title string
}

func newUIState() *uiState {
	return &uiState{
		symbolCache:  make(map[string]data.SymbolSnapshot),
		windowWidth:  120,
		windowHeight: 32,
		boxWidth:     4.0,
	}
}

func (s *uiState) observeSnapshot(snap data.Snapshot) {
	s.latest = snap
	for _, sym := range snap.Symbols {
		s.symbolCache[sym.Symbol] = sym
	}
	if len(s.watchlist) == 0 {
		for _, sym := range snap.Symbols {
			s.watchlist = append(s.watchlist, sym.Symbol)
		}
	} else {
		for _, sym := range snap.Symbols {
			if !s.hasSymbol(sym.Symbol) {
				s.watchlist = append(s.watchlist, sym.Symbol)
			}
		}
	}
}

func (s *uiState) hasSymbol(symbol string) bool {
	for _, existing := range s.watchlist {
		if strings.EqualFold(existing, symbol) {
			return true
		}
	}
	return false
}

func (s *uiState) addSymbol(symbol string) {
	cleaned := strings.ToUpper(strings.TrimSpace(symbol))
	if cleaned == "" {
		return
	}
	if !s.hasSymbol(cleaned) {
		s.watchlist = append(s.watchlist, cleaned)
	}
	if _, ok := s.symbolCache[cleaned]; !ok {
		s.symbolCache[cleaned] = data.SymbolSnapshot{Symbol: cleaned}
	}
}

func (s *uiState) currentOrder() []string {
	if s.dashboardOrder != nil && s.sortByMultiplier {
		return s.dashboardOrder
	}
	return s.watchlist
}

func (s *uiState) indexOfSymbol(symbol string) int {
	for idx, existing := range s.watchlist {
		if strings.EqualFold(existing, symbol) {
			return idx
		}
	}
	return -1
}

func (s *uiState) symbolData(symbol string) (data.SymbolSnapshot, bool) {
	sym, ok := s.symbolCache[symbol]
	if ok {
		return sym, true
	}
	for key, value := range s.symbolCache {
		if strings.EqualFold(key, symbol) {
			return value, true
		}
	}
	return data.SymbolSnapshot{Symbol: symbol}, false
}

func (s *uiState) setWindowSize(width, height int) bool {
	if width == s.windowWidth && height == s.windowHeight {
		return false
	}
	s.windowWidth = width
	s.windowHeight = height
	return true
}

func (s *uiState) sparklineWidth() int {
	switch {
	case s.windowWidth >= 160:
		return 32
	case s.windowWidth >= 130:
		return 26
	case s.windowWidth >= 110:
		return 20
	case s.windowWidth >= 90:
		return 16
	default:
		return 10
	}
}

func (s *uiState) optionRows(total int) int {
	if s.windowHeight <= 0 {
		return total
	}
	maxRows := s.windowHeight - 10
	if maxRows < 5 {
		maxRows = 5
	}
	if total < maxRows {
		return total
	}
	return maxRows
}

func (s *uiState) toggleSortByMultiplier() {
	s.sortByMultiplier = !s.sortByMultiplier
}

func (s *uiState) curveChartWidth() int {
	switch {
	case s.windowWidth >= 160:
		return 36
	case s.windowWidth >= 130:
		return 28
	case s.windowWidth >= 110:
		return 22
	default:
		return 16
	}
}

var mainTabDefinitions = []tabDefinition{
	{id: "dashboard", title: "Dashboard"},
	{id: "curve", title: "Yield Curve"},
	{id: "history", title: "Historical Data"},
	{id: "current", title: "Current Positions"},
	{id: "historic", title: "Historic Positions"},
	{id: "orders", title: "Orders"},
	{id: "alerts", title: "Alerts"},
	{id: "faq", title: "FAQs"},
}

// Run is entrypoint for TUI application.
func Run(ctx context.Context) error {
	return runWithOptions(ctx, runOptions{})
}

func runWithOptions(ctx context.Context, opts runOptions) error {
	app := tview.NewApplication()
	if opts.screen != nil {
		app = app.SetScreen(opts.screen)
	}

	ctx, cancel := context.WithCancel(ctx)
	defer cancel()

	state := newUIState()

	provider := opts.provider
	if provider == nil {
		backend := strings.ToLower(strings.TrimSpace(os.Getenv("TUI_BACKEND")))
		switch backend {
		case "", "mock":
			provider = newMockProvider(ctx)
			state.backendLabel = "Mock"
		case "rest":
			if endpoint := os.Getenv("TUI_API_URL"); endpoint != "" {
				rest := data.NewRestProvider(endpoint, 2*time.Second)
				rest.Start(ctx)
				provider = rest
				state.backendLabel = "REST"
			} else {
				provider = newMockProvider(ctx)
				state.backendLabel = "Mock"
			}
		case "nautilus", "nautilus_trader":
			provider = newNautilusProvider(ctx)
			state.backendLabel = "Nautilus Placeholder"
		default:
			log.Printf("[tui] unknown TUI_BACKEND=%s, falling back to mock provider", backend)
			provider = newMockProvider(ctx)
			state.backendLabel = "Mock"
		}
	} else {
		if state.backendLabel == "" {
			state.backendLabel = "Custom"
		}
	}
	if provider == nil {
		return fmt.Errorf("no data provider available")
	}
	if adder, ok := provider.(symbolAdder); ok {
		state.symbolAdder = adder
	}
	defer provider.Stop()

	header := buildHeader()
	dashboard := buildDashboard()
	yieldCurve := buildYieldCurve()
	historyTable := buildHistory()
	positions := buildPositions()
	historic := buildHistoric()
	orders := buildOrders()
	alerts := buildAlerts()
	controls := buildControls()
	faq := buildFAQ()
	updateControls(controls, state)

	tabs := buildTabs(mainTabDefinitions)

	tabContent := map[string]tview.Primitive{
		"dashboard": dashboard,
		"curve":     yieldCurve,
		"history":   historyTable,
		"current":   positions,
		"historic":  historic,
		"orders":    orders,
		"alerts":    alerts,
		"faq":       faq,
	}

	pages := tview.NewPages()
	for i, def := range mainTabDefinitions {
		page := tabContent[def.id]
		pages.AddPage(def.id, page, true, i == 0)
	}

	layout := tview.NewFlex().
		SetDirection(tview.FlexRow).
		AddItem(header, 3, 0, false).
		AddItem(tabs, 3, 0, false).
		AddItem(pages, 0, 1, true).
		AddItem(controls, 2, 0, false)

	focusTab := func(index int) {
		if index < 0 || index >= len(mainTabDefinitions) {
			return
		}
		switch mainTabDefinitions[index].id {
		case "dashboard":
			if state.dashboardTable != nil {
				app.SetFocus(state.dashboardTable)
			}
		case "curve":
			if state.yieldCurveTable != nil {
				app.SetFocus(state.yieldCurveTable)
			}
		case "history":
			if state.historyTable != nil {
				app.SetFocus(state.historyTable)
			}
		case "current":
			app.SetFocus(positions)
		case "historic":
			app.SetFocus(historic)
		case "orders":
			app.SetFocus(orders)
		case "alerts":
			app.SetFocus(alerts)
		case "faq":
			app.SetFocus(faq)
		}
	}

	tabs.SetChangedFunc(func(index int, mainText, secondaryText string, shortcut rune) {
		if index < 0 || index >= len(mainTabDefinitions) {
			return
		}
		pages.SwitchToPage(mainTabDefinitions[index].id)
		focusTab(index)
	})
	tabs.SetSelectedFunc(func(index int, mainText, secondaryText string, shortcut rune) {
		if index < 0 || index >= len(mainTabDefinitions) {
			return
		}
		pages.SwitchToPage(mainTabDefinitions[index].id)
		focusTab(index)
	})

	state.dashboardTable = extractDashboardTable(dashboard)
	if state.dashboardTable != nil {
		state.dashboardTable.SetSelectable(true, false)
		state.dashboardTable.SetFixed(1, 0)
	}

	state.dashboardPositions = extractDashboardPositions(dashboard)
	if state.dashboardPositions != nil {
		state.dashboardPositions.SetSelectable(true, false)
		state.dashboardPositions.SetFixed(1, 0)
	}

	state.dashboardCurve = extractDashboardCurve(dashboard)
	if state.dashboardCurve != nil {
		state.dashboardCurve.SetFixed(1, 0)
	}

	state.dashboardHistory = extractDashboardHistory(dashboard)
	if state.dashboardHistory != nil {
		state.dashboardHistory.SetFixed(1, 0)
	}

	state.dashboardOrders = extractDashboardOrders(dashboard)
	state.dashboardAlerts = extractDashboardAlerts(dashboard)

	state.historyTable = historyTable
	state.yieldCurveTable = yieldCurve
	state.faqView = faq

	if state.dashboardHistory != nil {
		state.dashboardHistory.SetSelectable(true, false)
		state.dashboardHistory.SetFixed(1, 0)
	}

	updateDashboardPreviews(state, state.latest)

	positions.SetSelectable(true, false)
	positions.SetFixed(1, 0)

	historic.SetSelectable(true, false)
	historic.SetFixed(1, 0)

	if state.historyTable != nil {
		state.historyTable.SetSelectable(true, false)
		state.historyTable.SetFixed(1, 0)
	}

	if yieldTable := state.yieldCurveTable; yieldTable != nil {
		yieldTable.SetSelectable(true, false)
		yieldTable.SetFixed(1, 0)
	}

	if state.dashboardTable != nil {
		state.dashboardTable.SetSelectedFunc(func(row, column int) {
			if row <= 0 {
				return
			}
			index := row - 1
			order := state.currentOrder()
			if index >= len(order) {
				return
			}
			symbol := order[index]
			showOptionChain(app, layout, state, symbol)
		})
	}

	if state.dashboardPositions != nil {
		state.dashboardPositions.SetSelectedFunc(func(row, column int) {
			if row <= 0 {
				return
			}
			index := row - 1
			if index >= len(state.latest.Positions) {
				return
			}
			showPositionDetail(app, layout, state.latest.Positions[index])
		})
	}

	if state.dashboardHistory != nil {
		state.dashboardHistory.SetSelectedFunc(func(row, column int) {
			if row <= 0 {
				return
			}
			index := row - 1
			if index >= len(state.latest.History) {
				return
			}
			showHistoricalDetail(app, layout, state.latest.History[index])
		})
	}

	positions.SetSelectedFunc(func(row, column int) {
		if row <= 0 {
			return
		}
		if row-1 >= len(state.latest.Positions) {
			return
		}
		pos := state.latest.Positions[row-1]
		showPositionDetail(app, layout, pos)
	})

	historic.SetSelectedFunc(func(row, column int) {
		if row <= 0 {
			return
		}
		if row-1 >= len(state.latest.Historic) {
			return
		}
		pos := state.latest.Historic[row-1]
		showPositionDetail(app, layout, pos)
	})

	app.SetRoot(layout, true)
	app.ForceDraw()
	app.SetFocus(layout)

	app.SetBeforeDrawFunc(func(screen tcell.Screen) bool {
		width, height := screen.Size()
		if state.setWindowSize(width, height) {
			app.QueueUpdateDraw(func() {
				updateControls(controls, state)
				updateDashboard(state.dashboardTable, state)
				updateDashboardPositions(state.dashboardPositions, state.latest.Positions, state.sparklineWidth())
				updateDashboardPreviews(state, state.latest)
				updatePositions(positions, state.latest.Positions, "Current Positions", state.sparklineWidth())
				updatePositions(historic, state.latest.Historic, "Historic Positions", state.sparklineWidth())
				updateHistory(state.historyTable, state.latest.History)
				updateYieldCurve(state.yieldCurveTable, state.latest.YieldCurve, state.curveChartWidth())
				updateFAQ(state.faqView, state.latest.FAQs)
			})
		}
		return false
	})

	layout.SetInputCapture(func(event *tcell.EventKey) *tcell.EventKey {
		if event.Key() == tcell.KeyRune && event.Rune() == '/' && event.Modifiers()&tcell.ModShift != 0 {
			showHelpModal(app, layout, state)
			return nil
		}
		switch event.Key() {
		case tcell.KeyTAB:
			count := tabs.GetItemCount()
			if count == 0 {
				return event
			}
			next := (tabs.GetCurrentItem() + 1) % count
			tabs.SetCurrentItem(next)
			pages.SwitchToPage(mainTabDefinitions[next].id)
			focusTab(next)
			return nil
		case tcell.KeyBacktab:
			count := tabs.GetItemCount()
			if count == 0 {
				return event
			}
			idx := tabs.GetCurrentItem() - 1
			if idx < 0 {
				idx = count - 1
			}
			tabs.SetCurrentItem(idx)
			pages.SwitchToPage(mainTabDefinitions[idx].id)
			focusTab(idx)
			return nil
		}

		switch event.Rune() {
		case 'm', 'M':
			state.toggleSortByMultiplier()
			updateControls(controls, state)
			updateDashboard(state.dashboardTable, state)
			updateDashboardPositions(state.dashboardPositions, state.latest.Positions, state.sparklineWidth())
			updateDashboardPreviews(state, state.latest)
			return nil
		case '?':
			showHelpModal(app, layout, state)
			return nil
		case 'a', 'A', '+':
			promptAddSymbol(app, layout, state)
			return nil
		case 'q', 'Q':
			app.Stop()
			return nil
		case 'b', 'B':
			showModal(app, layout, "Attempting to buy combo (mock)...")
			return nil
		case 's':
			if event.Modifiers()&tcell.ModShift != 0 {
				showModal(app, layout, "Attempting to sell combo (mock)...")
				return nil
			}
		}
		return event
	})

	app.SetInputCapture(func(event *tcell.EventKey) *tcell.EventKey {
		if event.Key() == tcell.KeyRune {
			if event.Rune() == '?' || (event.Rune() == '/' && event.Modifiers()&tcell.ModShift != 0) {
				showHelpModal(app, layout, state)
				writeKeyboardBanner(opts.screen)
				return nil
			}
			if strings.EqualFold(string(event.Rune()), "q") {
				clearKeyboardBanner(opts.screen)
			}
		}
		return event
	})

	go func() {
		for snap := range provider.Snapshots() {
			snapshot := snap
			app.QueueUpdateDraw(func() {
				state.helpBannerExpiry = time.Now().Add(750 * time.Millisecond)
				state.observeSnapshot(snapshot)
				updateHeader(header, snapshot, state.backendLabel, state)
				updateDashboard(state.dashboardTable, state)
				updateDashboardPositions(state.dashboardPositions, snapshot.Positions, state.sparklineWidth())
				updateDashboardPreviews(state, snapshot)
				updatePositions(positions, snapshot.Positions, "Current Positions", state.sparklineWidth())
				updatePositions(historic, snapshot.Historic, "Historic Positions", state.sparklineWidth())
				updateOrders(orders, snapshot.Orders)
				updateAlerts(alerts, snapshot.Alerts)
				updateHistory(state.historyTable, snapshot.History)
				updateYieldCurve(state.yieldCurveTable, snapshot.YieldCurve, state.curveChartWidth())
				updateFAQ(state.faqView, snapshot.FAQs)
				updateControls(controls, state)
				time.AfterFunc(800*time.Millisecond, func() {
					app.QueueUpdateDraw(func() {
						if time.Now().After(state.helpBannerExpiry) {
							updateControls(controls, state)
						}
					})
				})
				if !state.helpShown {
					showHelpModal(app, layout, state)
				}
			})
		}
	}()

	// run with context cancellation support
	errc := make(chan error, 1)
	go func() {
		errc <- app.Run()
	}()

	select {
	case <-ctx.Done():
		// allow some time for graceful stop
		go func() {
			time.Sleep(100 * time.Millisecond)
			app.Stop()
		}()
		return ctx.Err()
	case err := <-errc:
		return err
	}
}

// Helper builders ----------------------------------------------------------

func buildHeader() *tview.Table {
	table := tview.NewTable().SetBorders(false)
	table.SetCell(0, 0, tview.NewTableCell("[cyan]IB Box Spread Terminal[-]    Time: [white]--:--:--[-]    Source: [gray]--[-]").SetExpansion(1))
	table.SetCell(1, 0, tview.NewTableCell("Mode: [green]DRY-RUN[-]   Strategy: [green]RUNNING[-]   Account: [cyan]DU123456[-]").SetExpansion(1))
	table.SetCell(2, 0, tview.NewTableCell("TWS: [green]OK[-]   ORATS: [green]ENABLED[-]   Portal: [green]OK[-]   QuestDB: [green]OK[-]").SetExpansion(1))
	return table
}

func buildTabs(defs []tabDefinition) *tview.List {
	list := tview.NewList()
	list.ShowSecondaryText(false)
	list.SetHighlightFullLine(true)
	list.SetWrapAround(true)
	list.SetBorder(true)
	for _, def := range defs {
		list.AddItem(def.title, "", 0, nil)
	}
	if list.GetItemCount() > 0 {
		list.SetCurrentItem(0)
	}
	return list
}

func buildDashboard() *tview.Flex {
	layout := tview.NewFlex().SetDirection(tview.FlexRow)
	symbols := tview.NewTable().SetBorders(false)
	symbols.SetBorder(true).SetTitle("Symbols")
	positions := tview.NewTable().SetBorders(false)
	positions.SetBorder(true).SetTitle("Current Positions")

	summary := tview.NewFlex().SetDirection(tview.FlexColumn)
	curve := tview.NewTable().SetBorders(false)
	curve.SetBorder(true).SetTitle("Yield Curve Preview")
	history := tview.NewTable().SetBorders(false)
	history.SetBorder(true).SetTitle("Historical Preview")

	right := tview.NewFlex().SetDirection(tview.FlexRow)
	orders := tview.NewList()
	orders.SetBorder(true).SetTitle("Orders Preview")
	alerts := tview.NewTextView().SetDynamicColors(true).SetScrollable(true)
	alerts.SetBorder(true).SetTitle("Alerts Preview")
	right.AddItem(orders, 0, 1, false)
	right.AddItem(alerts, 0, 1, false)

	summary.AddItem(curve, 0, 1, false)
	summary.AddItem(history, 0, 1, false)
	summary.AddItem(right, 0, 1, false)

	layout.AddItem(symbols, 0, 3, true)
	layout.AddItem(positions, 0, 2, false)
	layout.AddItem(summary, 0, 2, false)
	return layout
}

func buildYieldCurve() *tview.Table {
	table := tview.NewTable().SetBorders(false)
	table.SetBorder(true).SetTitle("Yield Curve (Mock)")
	return table
}

func buildHistory() *tview.Table {
	table := tview.NewTable().SetBorders(false)
	table.SetBorder(true).SetTitle("Historical Box Data")
	return table
}

func buildPositions() *tview.Table {
	table := tview.NewTable().SetBorders(false)
	table.SetBorder(true).SetTitle("Current Positions")
	return table
}

func buildHistoric() *tview.Table {
	table := tview.NewTable().SetBorders(false)
	table.SetBorder(true).SetTitle("Historic Positions")
	return table
}

func buildOrders() *tview.List {
	list := tview.NewList()
	list.SetBorder(true).SetTitle("Recent Orders")
	return list
}

func buildAlerts() *tview.TextView {
	text := tview.NewTextView().SetDynamicColors(true)
	text.SetBorder(true).SetTitle("Alerts")
	text.SetScrollable(true)
	return text
}

func buildControls() *tview.TextView {
	info := tview.NewTextView().SetDynamicColors(true)
	info.SetBorder(true)
	return info
}

func buildFAQ() *tview.TextView {
	text := tview.NewTextView().
		SetDynamicColors(true).
		SetWrap(true).
		SetScrollable(true)
	text.SetBorder(true).SetTitle("Box Spread FAQs")
	return text
}

func extractDashboardTable(layout *tview.Flex) *tview.Table {
	if layout == nil {
		return nil
	}
	if layout.GetItemCount() == 0 {
		return nil
	}
	if tbl, ok := layout.GetItem(0).(*tview.Table); ok {
		return tbl
	}
	return nil
}

func extractDashboardPositions(layout *tview.Flex) *tview.Table {
	if layout == nil {
		return nil
	}
	if layout.GetItemCount() < 2 {
		return nil
	}
	if tbl, ok := layout.GetItem(1).(*tview.Table); ok {
		return tbl
	}
	return nil
}

func extractDashboardSummary(layout *tview.Flex) *tview.Flex {
	if layout == nil {
		return nil
	}
	if layout.GetItemCount() < 3 {
		return nil
	}
	if summary, ok := layout.GetItem(2).(*tview.Flex); ok {
		return summary
	}
	return nil
}

func extractDashboardCurve(layout *tview.Flex) *tview.Table {
	if summary := extractDashboardSummary(layout); summary != nil && summary.GetItemCount() >= 1 {
		if tbl, ok := summary.GetItem(0).(*tview.Table); ok {
			return tbl
		}
	}
	return nil
}

func extractDashboardHistory(layout *tview.Flex) *tview.Table {
	if summary := extractDashboardSummary(layout); summary != nil && summary.GetItemCount() >= 2 {
		if tbl, ok := summary.GetItem(1).(*tview.Table); ok {
			return tbl
		}
	}
	return nil
}

func extractDashboardOrders(layout *tview.Flex) *tview.List {
	if summary := extractDashboardSummary(layout); summary != nil && summary.GetItemCount() >= 3 {
		if right, ok := summary.GetItem(2).(*tview.Flex); ok && right.GetItemCount() >= 1 {
			if list, ok := right.GetItem(0).(*tview.List); ok {
				return list
			}
		}
	}
	return nil
}

func extractDashboardAlerts(layout *tview.Flex) *tview.TextView {
	if summary := extractDashboardSummary(layout); summary != nil && summary.GetItemCount() >= 3 {
		if right, ok := summary.GetItem(2).(*tview.Flex); ok && right.GetItemCount() >= 2 {
			if view, ok := right.GetItem(1).(*tview.TextView); ok {
				return view
			}
		}
	}
	return nil
}

func showModal(app *tview.Application, root tview.Primitive, message string) {
	modal := tview.NewModal().
		SetText(fmt.Sprintf("%s\nPress Enter to continue", message)).
		AddButtons([]string{"OK"})
	modal.SetDoneFunc(func(buttonIndex int, buttonLabel string) {
		app.SetRoot(root, true)
	})
	app.SetRoot(modal, true)
}

func updateHeader(table *tview.Table, snap data.Snapshot, backend string, state *uiState) {
	table.Clear()
	banner := ""
	if state != nil && time.Now().Before(state.helpBannerExpiry) {
		banner = "    [yellow]Keyboard Shortcuts available (press ?)[-]"
	}
	table.SetCell(0, 0, tview.NewTableCell(
		fmt.Sprintf("[cyan]IB Box Spread Terminal[-]    Time: [white]%s[-]    Source: %s%s",
			snap.GeneratedAt.Format("15:04:05"),
			formatBackendLabel(backend),
			banner)).SetExpansion(1))
	table.SetCell(1, 0, tview.NewTableCell(
		fmt.Sprintf("Mode: [green]%s[-]   Strategy: [green]%s[-]   Account: [cyan]%s[-]",
			snap.Mode, snap.Strategy, snap.AccountID)).SetExpansion(1))
	table.SetCell(2, 0, tview.NewTableCell(
		fmt.Sprintf("TWS: %s   ORATS: %s   Portal: %s   QuestDB: %s",
			healthColor(snap.Metrics.TWSOK, "OK"),
			healthColor(snap.Metrics.ORATSOK, "Enabled"),
			healthColor(snap.Metrics.PortalOK, "OK"),
			healthColor(snap.Metrics.QuestDBOK, "OK"))).SetExpansion(1))
	table.SetCell(3, 0, tview.NewTableCell(
		fmt.Sprintf("NetLiq: [green]$%.2f[-]   BuyingPower: [green]$%.2f[-]   MarginReq: [yellow]$%.2f[-]   Commissions: [magenta]$%.2f[-]",
			snap.Metrics.NetLiq, snap.Metrics.BuyingPower, snap.Metrics.MarginRequirement, snap.Metrics.Commissions)).SetExpansion(1))
}

func updateDashboard(table *tview.Table, state *uiState) {
	if table == nil {
		return
	}

	table.Clear()

	if len(state.watchlist) == 0 {
		table.SetCell(0, 0, tview.NewTableCell("[gray]No symbols tracked yet. Press [A] to add one.[-]").SetAlign(tview.AlignLeft))
		return
	}

	orderedSymbols := append([]string(nil), state.watchlist...)
	if state.sortByMultiplier {
		sort.SliceStable(orderedSymbols, func(i, j int) bool {
			di, _ := state.symbolData(orderedSymbols[i])
			dj, _ := state.symbolData(orderedSymbols[j])
			if di.Multiplier == dj.Multiplier {
				return orderedSymbols[i] < orderedSymbols[j]
			}
			return di.Multiplier > dj.Multiplier
		})
		state.dashboardOrder = make([]string, len(orderedSymbols))
		copy(state.dashboardOrder, orderedSymbols)
	} else {
		state.dashboardOrder = nil
	}

	type columnSpec struct {
		title  string
		align  int
		render func(sym data.SymbolSnapshot) string
	}

	sparkWidth := state.sparklineWidth()
	width := state.windowWidth

	formatPrice := func(value float64) string {
		if value == 0 {
			return "--"
		}
		return fmt.Sprintf("%.2f", value)
	}

	columns := []columnSpec{
		{title: "Symbol", align: tview.AlignLeft, render: func(sym data.SymbolSnapshot) string {
			if sym.Symbol == "" {
				return "[gray]--[-]"
			}
			return fmt.Sprintf("[white]%s[-]", sym.Symbol)
		}},
		{title: "Last", align: tview.AlignRight, render: func(sym data.SymbolSnapshot) string {
			if sym.Last == 0 {
				return "--"
			}
			return fmt.Sprintf("%.2f", sym.Last)
		}},
		{title: "Bid", align: tview.AlignRight, render: func(sym data.SymbolSnapshot) string {
			if sym.Bid == 0 {
				return "--"
			}
			return fmt.Sprintf("%.2f", sym.Bid)
		}},
		{title: "Ask", align: tview.AlignRight, render: func(sym data.SymbolSnapshot) string {
			if sym.Ask == 0 {
				return "--"
			}
			return fmt.Sprintf("%.2f", sym.Ask)
		}},
	}

	columns = append(columns, columnSpec{title: "Spread", align: tview.AlignRight, render: func(sym data.SymbolSnapshot) string {
		return formatPrice(sym.Spread)
	}})

	if width >= 110 {
		columns = append(columns, columnSpec{title: "ROI%", align: tview.AlignRight, render: func(sym data.SymbolSnapshot) string {
			if sym.ROI == 0 {
				return "--"
			}
			return fmt.Sprintf("[green]%.2f[-]", sym.ROI)
		}})
	}

	if width >= 130 {
		columns = append(columns, columnSpec{title: "Mk/Tk", align: tview.AlignCenter, render: func(sym data.SymbolSnapshot) string {
			if sym.MakerCount == 0 && sym.TakerCount == 0 {
				return "--/--"
			}
			return fmt.Sprintf("[cyan]%d[-]/[magenta]%d[-]", sym.MakerCount, sym.TakerCount)
		}})
	}

	if width >= 100 {
		columns = append(columns, columnSpec{title: "Vol", align: tview.AlignRight, render: func(sym data.SymbolSnapshot) string {
			if sym.Volume == 0 {
				return "--"
			}
			return fmt.Sprintf("%d", sym.Volume)
		}})
	}

	if width >= 90 {
		columns = append(columns, columnSpec{title: "Range", align: tview.AlignLeft, render: func(sym data.SymbolSnapshot) string {
			return drawCandle(sym.Candle, sparkWidth)
		}})
	}

	for i, col := range columns {
		table.SetCell(0, i, tview.NewTableCell("[yellow]"+col.title+"[-]").SetAlign(tview.AlignCenter))
	}

	for row, symbol := range orderedSymbols {
		dataPoint, _ := state.symbolData(symbol)
		r := row + 1
		for colIndex, col := range columns {
			table.SetCell(r, colIndex, tview.NewTableCell(col.render(dataPoint)).SetAlign(col.align))
		}
	}
}

func updateDashboardPositions(table *tview.Table, positions []data.Position, sparkWidth int) {
	if table == nil {
		return
	}

	limit := 5
	if sparkWidth > 26 {
		limit = 7
	} else if sparkWidth > 20 {
		limit = 6
	}

	var subset []data.Position
	if len(positions) > limit {
		subset = make([]data.Position, limit)
		copy(subset, positions[:limit])
	} else {
		subset = make([]data.Position, len(positions))
		copy(subset, positions)
	}

	displayLimit := limit
	if len(subset) > 0 && len(subset) < limit {
		displayLimit = len(subset)
	}

	updatePositions(table, subset, fmt.Sprintf("Current Positions (Top %d)", displayLimit), sparkWidth)
}

func showDashboardTablePlaceholder(table *tview.Table, title, message string) {
	if table == nil {
		return
	}
	table.Clear()
	table.SetTitle(title)
	table.SetCell(0, 0, tview.NewTableCell(fmt.Sprintf("[gray]%s[-]", message)).SetAlign(tview.AlignLeft))
}

func showDashboardListPlaceholder(list *tview.List, title, message string) {
	if list == nil {
		return
	}
	list.Clear()
	list.SetTitle(title)
	list.AddItem(fmt.Sprintf("[gray]%s[-]", message), "", 0, nil)
}

func updateYieldCurvePreview(table *tview.Table, points []data.YieldCurvePoint, chartWidth, limit int) {
	if table == nil {
		return
	}
	subset := points
	if limit > 0 && len(points) > limit {
		subset = append([]data.YieldCurvePoint(nil), points[:limit]...)
	}
	updateYieldCurve(table, subset, chartWidth)
	if limit > 0 && len(points) > limit {
		table.SetTitle(fmt.Sprintf("Yield Curve Preview (Top %d)", limit))
	} else {
		table.SetTitle("Yield Curve Preview")
	}
}

func updateHistoryPreview(table *tview.Table, history []data.HistoryEntry, limit int) {
	if table == nil {
		return
	}
	subset := history
	if limit > 0 && len(history) > limit {
		subset = append([]data.HistoryEntry(nil), history[:limit]...)
	}
	updateHistory(table, subset)
	if limit > 0 && len(history) > limit {
		table.SetTitle(fmt.Sprintf("Historical Preview (Top %d)", limit))
	} else {
		table.SetTitle("Historical Preview")
	}
}

func updateOrdersPreview(list *tview.List, orders []data.Order, limit int) {
	if list == nil {
		return
	}
	list.Clear()
	list.SetTitle("Orders Preview")
	subset := orders
	if limit > 0 && len(orders) > limit {
		subset = orders[:limit]
	}
	if len(subset) == 0 {
		list.AddItem("[gray]No recent orders[-]", "", 0, nil)
		return
	}
	for _, order := range subset {
		color := severityColor(order.Severity)
		text := fmt.Sprintf("%s %s%s[-]", order.Timestamp.Format("15:04:05"), color, order.Text)
		list.AddItem(text, "", 0, nil)
	}
}

func updateAlertsPreview(view *tview.TextView, alerts []data.Alert, limit int) {
	if view == nil {
		return
	}
	view.Clear()
	view.SetTitle("Alerts Preview")
	subset := alerts
	if limit > 0 && len(alerts) > limit {
		subset = alerts[:limit]
	}
	if len(subset) == 0 {
		_, _ = fmt.Fprintln(view, "[gray]No alerts[-]")
		return
	}
	for _, alert := range subset {
		_, _ = fmt.Fprintf(
			view,
			"%s %s%s[-]\n",
			alert.Timestamp.Format("15:04:05"),
			severityColor(alert.Severity),
			alert.Text,
		)
	}
}

func updateDashboardPreviews(state *uiState, snap data.Snapshot) {
	if state == nil {
		return
	}
	const (
		minPreviewWidth = 120
		curveLimit      = 6
		historyLimit    = 5
		ordersLimit     = 5
		alertsLimit     = 6
	)
	show := state.windowWidth >= minPreviewWidth

	if state.dashboardCurve != nil {
		if show {
			updateYieldCurvePreview(state.dashboardCurve, snap.YieldCurve, state.curveChartWidth(), curveLimit)
		} else {
			showDashboardTablePlaceholder(state.dashboardCurve, "Yield Curve Preview", "Increase terminal width to show preview")
		}
	}
	if state.dashboardHistory != nil {
		if show {
			updateHistoryPreview(state.dashboardHistory, snap.History, historyLimit)
		} else {
			showDashboardTablePlaceholder(state.dashboardHistory, "Historical Preview", "Increase terminal width to show preview")
		}
	}
	if state.dashboardOrders != nil {
		if show {
			updateOrdersPreview(state.dashboardOrders, snap.Orders, ordersLimit)
		} else {
			showDashboardListPlaceholder(state.dashboardOrders, "Orders Preview", "Increase terminal width to show preview")
		}
	}
	if state.dashboardAlerts != nil {
		if show {
			updateAlertsPreview(state.dashboardAlerts, snap.Alerts, alertsLimit)
		} else {
			state.dashboardAlerts.SetTitle("Alerts Preview")
			state.dashboardAlerts.SetText("[gray]Increase terminal width to show preview.[-]")
		}
	}
}

func updatePositions(table *tview.Table, positions []data.Position, title string, sparkWidth int) {
	if table == nil {
		return
	}

	table.Clear()
	table.SetTitle(title)

	headers := []string{"Name", "Qty", "ROI%", "Mk/Tk", "Rebate", "Vega", "Theta", "FairΔ", "Range"}
	for i, h := range headers {
		table.SetCell(0, i, tview.NewTableCell("[yellow]"+h+"[-]").SetAlign(tview.AlignCenter))
	}

	for row, pos := range positions {
		r := row + 1
		table.SetCell(r, 0, tview.NewTableCell(pos.Name))
		table.SetCell(r, 1, tview.NewTableCell(fmt.Sprintf("%d", pos.Quantity)).SetAlign(tview.AlignRight))
		table.SetCell(r, 2, tview.NewTableCell(fmt.Sprintf("[green]%.2f[-]", pos.ROI)).SetAlign(tview.AlignRight))
		table.SetCell(r, 3, tview.NewTableCell(fmt.Sprintf("[cyan]%d[-]/[magenta]%d[-]", pos.MakerCount, pos.TakerCount)).SetAlign(tview.AlignCenter))
		table.SetCell(r, 4, tview.NewTableCell(fmt.Sprintf("$%.2f", pos.RebateEstimate)).SetAlign(tview.AlignRight))
		table.SetCell(r, 5, tview.NewTableCell(fmt.Sprintf("%.3f", pos.Vega)).SetAlign(tview.AlignRight))
		table.SetCell(r, 6, tview.NewTableCell(fmt.Sprintf("%.3f", pos.Theta)).SetAlign(tview.AlignRight))
		table.SetCell(r, 7, tview.NewTableCell(fmt.Sprintf("%.3f", pos.FairDiff)).SetAlign(tview.AlignRight))
		table.SetCell(r, 8, tview.NewTableCell(drawCandle(pos.Candle, sparkWidth)).SetAlign(tview.AlignLeft))
	}
}

func updateOrders(list *tview.List, orders []data.Order) {
	if list == nil {
		return
	}
	list.Clear()
	for _, order := range orders {
		color := severityColor(order.Severity)
		text := fmt.Sprintf("%s %s%s[-]", order.Timestamp.Format("15:04:05"), color, order.Text)
		list.AddItem(text, "", 0, nil)
	}
}

func updateAlerts(text *tview.TextView, alerts []data.Alert) {
	if text == nil {
		return
	}
	text.Clear()
	for _, alert := range alerts {
		_, _ = fmt.Fprintf(text, "%s %s%s[-]\n", alert.Timestamp.Format("15:04:05"), severityColor(alert.Severity), alert.Text)
	}
}

func updateHistory(table *tview.Table, history []data.HistoryEntry) {
	if table == nil {
		return
	}

	table.Clear()
	table.SetTitle("Historical Box Data")

	headers := []struct {
		title string
		align int
	}{
		{"Date", tview.AlignLeft},
		{"Symbol", tview.AlignLeft},
		{"Expiry", tview.AlignLeft},
		{"DTE", tview.AlignCenter},
		{"Width", tview.AlignRight},
		{"Debit", tview.AlignRight},
		{"APR%", tview.AlignRight},
		{"Benchmark", tview.AlignLeft},
		{"Spread", tview.AlignRight},
		{"Notes", tview.AlignLeft},
	}

	for idx, col := range headers {
		table.SetCell(0, idx, tview.NewTableCell("[yellow]"+col.title+"[-]").SetAlign(col.align))
	}

	if len(history) == 0 {
		table.SetCell(1, 0, tview.NewTableCell("[gray]No historical records yet (mock data).[-]").SetAlign(tview.AlignLeft))
		return
	}

	for row, entry := range history {
		r := row + 1
		table.SetCell(r, 0, tview.NewTableCell(entry.Date.Format("2006-01-02")))
		table.SetCell(r, 1, tview.NewTableCell(entry.Symbol))
		table.SetCell(r, 2, tview.NewTableCell(entry.Expiration))
		table.SetCell(r, 3, tview.NewTableCell(fmt.Sprintf("%.0f", entry.DaysToExpiry)).SetAlign(tview.AlignCenter))
		table.SetCell(r, 4, tview.NewTableCell(fmt.Sprintf("%.2f", entry.Width)).SetAlign(tview.AlignRight))
		table.SetCell(r, 5, tview.NewTableCell(fmt.Sprintf("$%.2f", entry.NetDebit)).SetAlign(tview.AlignRight))
		table.SetCell(r, 6, tview.NewTableCell(fmt.Sprintf("%.2f", entry.APR)).SetAlign(tview.AlignRight))
		benchmark := fmt.Sprintf("%s %.2f%%", entry.Benchmark, entry.BenchmarkRate)
		table.SetCell(r, 7, tview.NewTableCell(benchmark))
		spread := entry.APR - entry.BenchmarkRate
		spreadCell := tview.NewTableCell(formatAPRSpread(spread)).SetAlign(tview.AlignRight)
		table.SetCell(r, 8, spreadCell)
		table.SetCell(r, 9, tview.NewTableCell(entry.Notes))
	}
}

func updateYieldCurve(table *tview.Table, points []data.YieldCurvePoint, chartWidth int) {
	if table == nil {
		return
	}

	table.Clear()
	table.SetTitle("Yield Curve (Mock)")

	headers := []struct {
		title string
		align int
	}{
		{"Label", tview.AlignLeft},
		{"Expiry", tview.AlignLeft},
		{"DTE", tview.AlignCenter},
		{"Debit", tview.AlignRight},
		{"APR%", tview.AlignRight},
		{"Benchmark%", tview.AlignRight},
		{"Spread", tview.AlignRight},
		{"Curve", tview.AlignLeft},
	}

	for idx, col := range headers {
		table.SetCell(0, idx, tview.NewTableCell("[yellow]"+col.title+"[-]").SetAlign(col.align))
	}

	if len(points) == 0 {
		table.SetCell(1, 0, tview.NewTableCell("[gray]No curve data yet (mock data).[-]").SetAlign(tview.AlignLeft))
		return
	}

	minAPR := math.MaxFloat64
	maxAPR := -math.MaxFloat64
	for _, pt := range points {
		if pt.APR < minAPR {
			minAPR = pt.APR
		}
		if pt.Benchmark < minAPR {
			minAPR = pt.Benchmark
		}
		if pt.APR > maxAPR {
			maxAPR = pt.APR
		}
		if pt.Benchmark > maxAPR {
			maxAPR = pt.Benchmark
		}
	}
	if minAPR == math.MaxFloat64 {
		minAPR = 0
	}
	if maxAPR <= minAPR {
		maxAPR = minAPR + 1
	}

	for row, pt := range points {
		r := row + 1
		table.SetCell(r, 0, tview.NewTableCell(pt.Label))
		table.SetCell(r, 1, tview.NewTableCell(pt.Expiration))
		table.SetCell(r, 2, tview.NewTableCell(fmt.Sprintf("%.0f", pt.DTE)).SetAlign(tview.AlignCenter))
		table.SetCell(r, 3, tview.NewTableCell(fmt.Sprintf("$%.2f", pt.NetDebit)).SetAlign(tview.AlignRight))
		table.SetCell(r, 4, tview.NewTableCell(fmt.Sprintf("%.2f", pt.APR)).SetAlign(tview.AlignRight))
		table.SetCell(r, 5, tview.NewTableCell(fmt.Sprintf("%.2f", pt.Benchmark)).SetAlign(tview.AlignRight))
		table.SetCell(r, 6, tview.NewTableCell(formatAPRSpread(pt.APRSpread)).SetAlign(tview.AlignRight))
		bar := drawAPRBar(pt.APR, pt.Benchmark, minAPR, maxAPR, chartWidth)
		table.SetCell(r, 7, tview.NewTableCell(bar))
	}
}

func updateFAQ(view *tview.TextView, faqs []data.FAQEntry) {
	if view == nil {
		return
	}
	var builder strings.Builder
	if len(faqs) == 0 {
		builder.WriteString("[gray]No FAQs loaded (mock data).[-]")
		view.SetText(builder.String())
		return
	}
	for idx, faq := range faqs {
		if idx > 0 {
			builder.WriteString("\n")
		}
		builder.WriteString(fmt.Sprintf("[yellow]Q: %s[-]\n", faq.Question))
		builder.WriteString(fmt.Sprintf("   A: %s\n", faq.Answer))
	}
	view.SetText(builder.String())
}

func updateControls(view *tview.TextView, state *uiState) {
	if view == nil || state == nil {
		return
	}

	width := state.windowWidth
	sortMode := "natural"
	if state.sortByMultiplier {
		sortMode = "multiplier"
	}
	banner := ""
	if time.Now().Before(state.helpBannerExpiry) {
		banner = "[yellow]Keyboard Shortcuts[-] press ?   "
	}
	var text string
	switch {
	case width >= 130:
		text = fmt.Sprintf("%sControls: [cyan][Tab/Shift+Tab][-] tabs  [green][Enter][-] option chain  [magenta][A or +][-] add symbol  [blue][M][-] sort(%s)  [yellow][?][-] help  [blue][B][-] buy mock  [blue][Shift+S][-] sell mock  [red][Q][-] quit", banner, sortMode)
	case width >= 100:
		text = fmt.Sprintf("%sControls: [cyan]Tab[-]/[cyan]Shift+Tab[-] tabs  [green]Enter[-] chain  [magenta]A[-] add  [blue]M[-] sort(%s)  [yellow]?[-] help  [red]Q[-] quit", banner, sortMode)
	default:
		text = fmt.Sprintf("%sControls: Tab • Enter • A • M(%s) • ? • Q", banner, sortMode)
	}
	view.SetText(text)
}

func promptAddSymbol(app *tview.Application, root tview.Primitive, state *uiState) {
	if app == nil {
		return
	}

	input := tview.NewInputField().SetLabel("Symbol: ").SetFieldWidth(12)
	status := tview.NewTextView().SetDynamicColors(true).SetText("[gray]Symbols are uppercased automatically.[-]")

	form := tview.NewForm().
		AddFormItem(input).
		AddButton("Add", func() {
			symbol := strings.ToUpper(strings.TrimSpace(input.GetText()))
			if symbol == "" {
				status.SetText("[red]Enter a symbol first.[-]")
				return
			}
			if state.hasSymbol(symbol) {
				status.SetText(fmt.Sprintf("[yellow]%s is already tracked.[-]", symbol))
				return
			}
			state.addSymbol(symbol)
			if state.symbolAdder != nil {
				go func(sym string) {
					if err := state.symbolAdder.AddSymbol(sym); err != nil {
						log.Printf("[tui] add symbol %s failed: %v", sym, err)
					}
				}(symbol)
			}
			updateDashboard(state.dashboardTable, state)
			if state.dashboardTable != nil {
				idx := state.indexOfSymbol(symbol)
				if idx >= 0 {
					state.dashboardTable.Select(idx+1, 0)
				}
			}
			app.SetRoot(root, true)
			if state.dashboardTable != nil {
				app.SetFocus(state.dashboardTable)
			}
		})
	form.AddButton("Cancel", func() {
		app.SetRoot(root, true)
		if state.dashboardTable != nil {
			app.SetFocus(state.dashboardTable)
		}
	})
	form.SetCancelFunc(func() {
		app.SetRoot(root, true)
		if state.dashboardTable != nil {
			app.SetFocus(state.dashboardTable)
		}
	})

	dialog := tview.NewFlex().SetDirection(tview.FlexRow).
		AddItem(form, 0, 1, true).
		AddItem(status, 2, 0, false)

	frame := tview.NewFrame(dialog).
		SetTitle("Add Symbol").
		SetBorder(true)

	app.SetRoot(frame, true)
	app.SetFocus(form)
}

func showHelpModal(app *tview.Application, root tview.Primitive, state *uiState) {
	if state != nil {
		state.helpShown = true
	}
	help := tview.NewTextView().
		SetDynamicColors(true).
		SetText("[yellow]Keyboard Shortcuts[-]\n\n?  Show this help\nA/+  Add symbol to dashboard\nEnter  Open option chain for selection\nM  Toggle multiplier sort\nTab / Shift+Tab  Cycle tabs\nB  Mock buy combo\nShift+S  Mock sell combo\nQ  Quit application").
		SetWrap(true)
	help.SetBorder(true).SetTitle("Help (top-inspired)")

	footer := tview.NewTextView().
		SetDynamicColors(true).
		SetText("[gray]Press Esc or Q to return.[-]").
		SetTextAlign(tview.AlignCenter)

	layout := tview.NewFlex().SetDirection(tview.FlexRow).
		AddItem(help, 0, 1, true).
		AddItem(footer, 1, 0, false)

	close := func() {
		app.SetRoot(root, true)
		if state.dashboardTable != nil {
			app.SetFocus(state.dashboardTable)
		}
	}

	help.SetDoneFunc(func(key tcell.Key) {
		close()
	})
	help.SetInputCapture(func(event *tcell.EventKey) *tcell.EventKey {
		switch event.Key() {
		case tcell.KeyEsc:
			close()
			return nil
		}
		switch event.Rune() {
		case 'q', 'Q':
			close()
			return nil
		}
		return event
	})

	app.SetRoot(layout, true)
	app.SetFocus(help)
	app.Draw()
}

func showSymbolDetail(app *tview.Application, root tview.Primitive, sym data.SymbolSnapshot) {
	rangeWidth := sym.Candle.High - sym.Candle.Low
	closeRank := 0.0
	entryRank := 0.0
	if rangeWidth > 0 {
		closeRank = (sym.Candle.Close - sym.Candle.Low) / rangeWidth * 100
		entryRank = (sym.Candle.Entry - sym.Candle.Low) / rangeWidth * 100
	}

	lines := []string{
		fmt.Sprintf("Last: %.2f   Bid: %.2f   Ask: %.2f   Spread: %.2f", sym.Last, sym.Bid, sym.Ask, sym.Spread),
		fmt.Sprintf("ROI: %.2f%%   Maker/Taker: %d/%d   Volume: %d", sym.ROI, sym.MakerCount, sym.TakerCount, sym.Volume),
		fmt.Sprintf("Day Range: %.2f – %.2f", sym.Candle.Low, sym.Candle.High),
		fmt.Sprintf("Close Rank: %.1f%%   Entry Rank: %.1f%%", closeRank, entryRank),
	}

	showDetailModal(app, root, sym.Symbol, lines)
}

func showPositionDetail(app *tview.Application, root tview.Primitive, pos data.Position) {
	lines := []string{
		fmt.Sprintf("Quantity: %d    ROI: %.2f%%", pos.Quantity, pos.ROI),
		fmt.Sprintf("Maker/Taker: %d/%d    Rebate: $%.2f", pos.MakerCount, pos.TakerCount, pos.RebateEstimate),
		fmt.Sprintf("Vega: %.4f    Theta: %.4f", pos.Vega, pos.Theta),
		fmt.Sprintf("Fair Value Δ: %.4f", pos.FairDiff),
		fmt.Sprintf("Combo Range: %.2f – %.2f", pos.Candle.Low, pos.Candle.High),
	}

	showDetailModal(app, root, pos.Name, lines)
}

func showHistoricalDetail(app *tview.Application, root tview.Primitive, entry data.HistoryEntry) {
	lines := []string{
		fmt.Sprintf("Symbol: %s    Expiry: %s (%.0f DTE)", entry.Symbol, entry.Expiration, entry.DaysToExpiry),
		fmt.Sprintf("Width: %.2f    Net Debit: $%.2f", entry.Width, entry.NetDebit),
		fmt.Sprintf("APR: %.2f%%    Benchmark: %s %.2f%%", entry.APR, entry.Benchmark, entry.BenchmarkRate),
		fmt.Sprintf("APR vs Benchmark: %.2f%%", entry.APR-entry.BenchmarkRate),
	}
	if strings.TrimSpace(entry.Notes) != "" {
		lines = append(lines, fmt.Sprintf("Notes: %s", entry.Notes))
	}
	showDetailModal(app, root, fmt.Sprintf("%s Historical Box", entry.Symbol), lines)
}

func showDetailModal(app *tview.Application, root tview.Primitive, title string, lines []string) {
	body := fmt.Sprintf("[yellow]%s[-]\n\n%s", title, strings.Join(lines, "\n"))
	modal := tview.NewModal().
		SetText(body).
		AddButtons([]string{"OK"})
	modal.SetDoneFunc(func(buttonIndex int, buttonLabel string) {
		app.SetRoot(root, true)
	})
	app.SetRoot(modal, true)
}

func showOptionChain(app *tview.Application, root tview.Primitive, state *uiState, symbol string) {
	sym, _ := state.symbolData(symbol)
	if len(sym.OptionChains) == 0 {
		showSymbolDetail(app, root, sym)
		return
	}

	type seriesEntry struct {
		series    data.OptionSeries
		expiry    time.Time
		hasExpiry bool
	}

	entries := make([]seriesEntry, 0, len(sym.OptionChains))
	for _, series := range sym.OptionChains {
		entry := seriesEntry{series: series}
		if t, err := time.Parse("2006-01-02", series.Expiration); err == nil {
			entry.expiry = t
			entry.hasExpiry = true
		}
		entries = append(entries, entry)
	}
	if len(entries) == 0 {
		showSymbolDetail(app, root, sym)
		return
	}

	sort.SliceStable(entries, func(i, j int) bool {
		a, b := entries[i], entries[j]
		switch {
		case a.hasExpiry && b.hasExpiry:
			return a.expiry.Before(b.expiry)
		case a.hasExpiry:
			return true
		case b.hasExpiry:
			return false
		default:
			return a.series.Expiration < b.series.Expiration
		}
	})

	summary := tview.NewTextView().SetDynamicColors(true).SetWrap(true)
	table := tview.NewTable().SetBorders(false)
	footer := tview.NewTextView().
		SetDynamicColors(true).
		SetText("[gray]←/→ change expiry  •  E enter expiry/DTE  •  B box detail  •  W box span  •  Esc or Q return[-]").
		SetTextAlign(tview.AlignCenter)

	container := tview.NewFlex().SetDirection(tview.FlexRow).
		AddItem(summary, 3, 0, false).
		AddItem(table, 0, 1, true).
		AddItem(footer, 1, 0, false)

	refTime := state.latest.GeneratedAt
	if refTime.IsZero() {
		refTime = time.Now()
	}

	currentIndex := 0
	var currentStrikes []data.OptionStrike
	currentStart := 0
	var currentSeries data.OptionSeries

	renderSeries := func(index int) {
		if index < 0 || index >= len(entries) {
			return
		}
		currentIndex = index
		entry := entries[index]
		series := entry.series
		currentSeries = series

		strikes := append([]data.OptionStrike(nil), series.Strikes...)
		sort.Slice(strikes, func(i, j int) bool {
			return strikes[i].Strike < strikes[j].Strike
		})
		if len(strikes) == 0 {
			table.Clear()
			table.SetBorder(true).SetTitle(fmt.Sprintf("%s %s Option Chain", sym.Symbol, series.Expiration))
			table.SetSelectable(false, false)
			summary.SetText(fmt.Sprintf("[white]%s[-] Last %.2f  Bid %.2f  Ask %.2f  Spread %.2f  ROI %.2f%%\n[gray]No strikes available[-]",
				sym.Symbol, sym.Last, sym.Bid, sym.Ask, sym.Spread, sym.ROI))
			currentStrikes = nil
			currentStart = 0
			return
		}

		rows := state.optionRows(len(strikes))
		atmIndex := 0
		minDiff := math.MaxFloat64
		for i, strike := range strikes {
			diff := math.Abs(strike.Strike - sym.Last)
			if diff < minDiff {
				minDiff = diff
				atmIndex = i
			}
		}

		start := atmIndex - rows/2
		if start < 0 {
			start = 0
		}
		end := start + rows
		if end > len(strikes) {
			end = len(strikes)
			start = end - rows
			if start < 0 {
				start = 0
			}
		}
		visible := strikes[start:end]
		currentStrikes = strikes
		currentStart = start

		table.Clear()
		table.SetBorder(true).SetTitle(fmt.Sprintf("%s %s Option Chain", sym.Symbol, series.Expiration))

		headers := []string{"Call Bid", "Call Ask", "Strike", "Put Bid", "Put Ask"}
		for col, header := range headers {
			align := tview.AlignRight
			if header == "Strike" {
				align = tview.AlignCenter
			}
			table.SetCell(0, col, tview.NewTableCell("[yellow]"+header+"[-]").SetAlign(align))
		}

		for row, strike := range visible {
			r := row + 1
			table.SetCell(r, 0, tview.NewTableCell(fmt.Sprintf("%.2f", strike.CallBid)).SetAlign(tview.AlignRight))
			table.SetCell(r, 1, tview.NewTableCell(fmt.Sprintf("%.2f", strike.CallAsk)).SetAlign(tview.AlignRight))
			table.SetCell(r, 2, tview.NewTableCell(fmt.Sprintf("%.2f", strike.Strike)).SetAlign(tview.AlignCenter))
			table.SetCell(r, 3, tview.NewTableCell(fmt.Sprintf("%.2f", strike.PutBid)).SetAlign(tview.AlignRight))
			table.SetCell(r, 4, tview.NewTableCell(fmt.Sprintf("%.2f", strike.PutAsk)).SetAlign(tview.AlignRight))
		}

		selectedRow := atmIndex - start + 1
		if selectedRow < 1 {
			selectedRow = 1
		}
		if selectedRow >= table.GetRowCount() {
			selectedRow = table.GetRowCount() - 1
		}
		if selectedRow < 1 {
			selectedRow = 1
		}

		table.SetFixed(1, 0)
		table.SetSelectable(true, false)
		table.Select(selectedRow, 0)

		metrics := computeBoxMetrics(state, sym, series)
		summaryText := fmt.Sprintf("[white]%s[-] Last %.2f  Bid %.2f  Ask %.2f  Spread %.2f  ROI %.2f%%",
			sym.Symbol, sym.Last, sym.Bid, sym.Ask, sym.Spread, sym.ROI)
		summaryText += fmt.Sprintf("\nContract Multiplier: %.0f", sym.Multiplier)

		if entry.hasExpiry {
			dte := entry.expiry.Sub(refTime).Hours() / 24.0
			if dte < 0 {
				dte = 0
			}
			summaryText += fmt.Sprintf("\nExpiry: %s (%.0f DTE)", series.Expiration, dte)
		} else {
			summaryText += fmt.Sprintf("\nExpiry: %s", series.Expiration)
		}

		if metrics != nil {
			summaryText += fmt.Sprintf(
				"\nBox %.2f / %.2f (width %.2f)\nDebit: $%.2f  Payoff: $%.2f  Profit: $%.2f (%.2f%%)\nAPR: %.2f%%  Benchmark: %s %.2f%%  APR vs %s: %.2f%%",
				metrics.lowerStrike,
				metrics.upperStrike,
				metrics.width,
				metrics.cashFlow,
				metrics.payoff,
				metrics.profit,
				metrics.profitPct,
				metrics.aprPct,
				metrics.benchmark.symbol,
				metrics.benchmark.ratePct,
				metrics.benchmark.symbol,
				metrics.aprVsBenchmark,
			)
		}
		summaryText += fmt.Sprintf("\nBox span setting: %.1f strikes", state.boxWidth)
		summaryText += "\n[gray]Use ←/→ keys, B to open box spread, W to change span, or 'E' to jump to a specific expiry."
		summary.SetText(summaryText)
	}

	findClosestByDate := func(target time.Time) int {
		bestIdx := -1
		bestDiff := math.MaxFloat64
		for idx, entry := range entries {
			switch {
			case entry.hasExpiry:
				diff := math.Abs(entry.expiry.Sub(target).Hours())
				if diff < bestDiff {
					bestDiff = diff
					bestIdx = idx
				}
			default:
				if entry.series.Expiration == target.Format("2006-01-02") {
					return idx
				}
			}
		}
		return bestIdx
	}

	findClosestByDTE := func(target float64) int {
		bestIdx := -1
		bestDiff := math.MaxFloat64
		for idx, entry := range entries {
			if !entry.hasExpiry {
				continue
			}
			dte := entry.expiry.Sub(refTime).Hours() / 24.0
			diff := math.Abs(dte - target)
			if diff < bestDiff {
				bestDiff = diff
				bestIdx = idx
			}
		}
		return bestIdx
	}

	renderSeries(currentIndex)

	close := func() {
		app.SetRoot(root, true)
		if state.dashboardTable != nil {
			app.SetFocus(state.dashboardTable)
		}
	}

	restore := func() {
		app.SetRoot(container, true)
		app.SetFocus(table)
	}

	var showBoxDetail func(idx int)
	var promptBoxWidth func(idx int)

	showBoxDetail = func(idx int) {
		if currentStrikes == nil || idx < 0 || idx >= len(currentStrikes) {
			return
		}
		targetIdx := idx

		span := int(math.Max(1, math.Round(state.boxWidth)))
		if span > len(currentStrikes) {
			span = len(currentStrikes)
		}
		lowerIdx := targetIdx - span/2
		upperIdx := targetIdx + span/2
		if lowerIdx < 0 {
			lowerIdx = 0
		}
		if upperIdx >= len(currentStrikes) {
			upperIdx = len(currentStrikes) - 1
		}
		if lowerIdx == upperIdx {
			if upperIdx < len(currentStrikes)-1 {
				upperIdx++
			} else if lowerIdx > 0 {
				lowerIdx--
			}
		}

		lower := currentStrikes[lowerIdx]
		upper := currentStrikes[upperIdx]
		if upper.Strike <= lower.Strike {
			return
		}

		multiplier := sym.Multiplier
		if multiplier <= 0 {
			multiplier = 100
		}

		debit := lower.CallAsk - upper.CallBid + upper.PutAsk - lower.PutBid
		cash := debit * multiplier
		payoff := (upper.Strike - lower.Strike) * multiplier
		profit := payoff - cash
		profitPct := 0.0
		if cash > 1e-6 {
			profitPct = (profit / cash) * 100.0
		}

		dte := 0.0
		if currentSeries.Expiration != "" {
			if expiry, err := time.Parse("2006-01-02", currentSeries.Expiration); err == nil {
				dte = expiry.Sub(refTime).Hours() / 24.0
				if dte < 0 {
					dte = 0
				}
			}
		}

		apr := 0.0
		if cash > 1e-6 && dte > 0 {
			apr = profitPct * (365.0 / dte)
		}

		benchmark := selectBenchmark(dte)
		diffAPR := apr - benchmark.ratePct

		spannedStrikes := upperIdx - lowerIdx + 1
		detailText := fmt.Sprintf(
			"[yellow]%s Box Spread[-]\n\nSelected Strike: %.2f  (span setting: %.1f strikes; covering %d legs)\nLower Strike: %.2f\nUpper Strike: %.2f\nWidth: %.2f\nDebit: $%.2f\nMax Payoff: $%.2f\nProfit: $%.2f (%.2f%%)\nAPR: %.2f%%\nBenchmark: %s %.2f%%\nAPR vs Benchmark: %.2f%%\n\n[gray]Use Width… to change the span, Close to return.[-]",
			sym.Symbol,
			currentStrikes[targetIdx].Strike,
			state.boxWidth,
			spannedStrikes,
			lower.Strike,
			upper.Strike,
			upper.Strike-lower.Strike,
			cash,
			payoff,
			profit,
			profitPct,
			apr,
			benchmark.symbol,
			benchmark.ratePct,
			diffAPR,
		)

		modal := tview.NewModal().
			SetText(detailText).
			AddButtons([]string{"Close", "Width..."})
		modal.SetDoneFunc(func(i int, l string) {
			switch l {
			case "Width...":
				promptBoxWidth(targetIdx)
			default:
				restore()
			}
		})
		app.SetRoot(modal, true)
	}

	promptBoxWidth = func(idx int) {
		input := tview.NewInputField().
			SetLabel("Box span (# strikes): ").
			SetFieldWidth(10).
			SetText(fmt.Sprintf("%.1f", state.boxWidth))
		status := tview.NewTextView().
			SetDynamicColors(true).
			SetText("[gray]Enter a value ≥ 1. Decimals will be rounded to the nearest strike count.[-]")

		form := tview.NewForm().
			AddFormItem(input).
			AddButton("Apply", func() {
				valText := strings.TrimSpace(input.GetText())
				widthVal, err := strconv.ParseFloat(valText, 64)
				if err != nil || widthVal < 1 {
					status.SetText("[red]Enter a numeric span of at least 1.[-]")
					return
				}
				maxWidth := float64(len(currentStrikes))
				if maxWidth < 1 {
					maxWidth = 1
				}
				if widthVal > maxWidth {
					widthVal = maxWidth
				}
				state.boxWidth = widthVal
				restore()
				showBoxDetail(idx)
			}).
			AddButton("Cancel", func() {
				restore()
			})
		form.SetCancelFunc(func() {
			restore()
		})

		dialog := tview.NewFlex().SetDirection(tview.FlexRow).
			AddItem(form, 0, 1, true).
			AddItem(status, 1, 0, false)

		app.SetRoot(dialog, true)
		app.SetFocus(form)
	}

	promptExpiryInput := func() {
		input := tview.NewInputField().SetLabel("Expiry/DTE: ").SetFieldWidth(20)
		status := tview.NewTextView().
			SetDynamicColors(true).
			SetText("[gray]Enter YYYY-MM-DD or number of days to expiry[-]")

		form := tview.NewForm().
			AddFormItem(input).
			AddButton("Go", func() {
				val := strings.TrimSpace(input.GetText())
				if val == "" {
					status.SetText("[red]Enter an expiry or DTE value[-]")
					return
				}
				if date, err := time.Parse("2006-01-02", val); err == nil {
					if idx := findClosestByDate(date); idx >= 0 {
						renderSeries(idx)
						restore()
						return
					}
					status.SetText("[red]No matching expiry found[-]")
					return
				}
				if days, err := strconv.ParseFloat(val, 64); err == nil {
					if idx := findClosestByDTE(days); idx >= 0 {
						renderSeries(idx)
						restore()
						return
					}
					status.SetText("[red]No matching expiry found[-]")
					return
				}
				status.SetText("[red]Enter YYYY-MM-DD or numeric DTE][-]")
			}).
			AddButton("Cancel", func() {
				restore()
			})
		form.SetCancelFunc(func() {
			restore()
		})

		dialog := tview.NewFlex().SetDirection(tview.FlexRow).
			AddItem(form, 0, 1, true).
			AddItem(status, 1, 0, false)

		app.SetRoot(dialog, true)
		app.SetFocus(form)
	}

	table.SetDoneFunc(func(key tcell.Key) {
		close()
	})
	table.SetInputCapture(func(event *tcell.EventKey) *tcell.EventKey {
		switch event.Key() {
		case tcell.KeyEsc:
			close()
			return nil
		case tcell.KeyLeft:
			if currentIndex > 0 {
				renderSeries(currentIndex - 1)
			}
			return nil
		case tcell.KeyRight:
			if currentIndex+1 < len(entries) {
				renderSeries(currentIndex + 1)
			}
			return nil
		}
		switch event.Rune() {
		case 'q', 'Q':
			close()
			return nil
		case 'e', 'E':
			promptExpiryInput()
			return nil
		case 'w', 'W':
			if currentStrikes == nil {
				return nil
			}
			row, _ := table.GetSelection()
			if row <= 0 {
				row = 1
			}
			idx := currentStart + (row - 1)
			if idx < 0 || idx >= len(currentStrikes) {
				return nil
			}
			promptBoxWidth(idx)
			return nil
		case 'b', 'B':
			if currentStrikes == nil {
				return nil
			}
			row, _ := table.GetSelection()
			if row <= 0 {
				return nil
			}
			idx := currentStart + (row - 1)
			if idx < 0 || idx >= len(currentStrikes) {
				return nil
			}
			showBoxDetail(idx)
			return nil
		}
		return event
	})

	app.SetRoot(container, true)
	app.SetFocus(table)
}

func healthColor(ok bool, label string) string {
	if ok {
		return fmt.Sprintf("[green]%s[-]", label)
	}
	return fmt.Sprintf("[red]%s[-]", label)
}

func severityColor(severity string) string {
	switch strings.ToLower(severity) {
	case "warn", "warning":
		return "[yellow]"
	case "error", "critical":
		return "[red]"
	case "success":
		return "[green]"
	default:
		return "[blue]"
	}
}

func formatBackendLabel(label string) string {
	trimmed := strings.TrimSpace(label)
	if trimmed == "" {
		return "[gray]--[-]"
	}
	switch strings.ToLower(trimmed) {
	case "mock":
		return "[yellow]Mock[-]"
	case "rest":
		return "[green]REST[-]"
	case "nautilus placeholder", "nautilus":
		return "[magenta]Nautilus Placeholder[-]"
	case "custom":
		return "[cyan]Custom[-]"
	default:
		return fmt.Sprintf("[cyan]%s[-]", trimmed)
	}
}

func formatAPRSpread(spread float64) string {
	if spread >= 0 {
		return fmt.Sprintf("[green]+%.2f%%[-]", spread)
	}
	return fmt.Sprintf("[red]%.2f%%[-]", spread)
}

func drawAPRBar(value, benchmark, min, max float64, width int) string {
	if width < 6 {
		width = 6
	}
	if max <= min {
		max = min + 1
	}

	scale := float64(width - 1)
	valuePos := int(math.Round((value - min) / (max - min) * scale))
	benchPos := int(math.Round((benchmark - min) / (max - min) * scale))

	if valuePos < 0 {
		valuePos = 0
	}
	if valuePos >= width {
		valuePos = width - 1
	}
	if benchPos < 0 {
		benchPos = 0
	}
	if benchPos >= width {
		benchPos = width - 1
	}

	var builder strings.Builder
	builder.Grow(width + 2)
	builder.WriteString("[")
	for i := 0; i < width; i++ {
		switch {
		case i == valuePos && i == benchPos:
			builder.WriteString("[cyan]|[-]")
		case i == valuePos:
			builder.WriteString("[green]|[-]")
		case i == benchPos:
			builder.WriteString("[yellow]|[-]")
		case i < valuePos:
			builder.WriteRune('=')
		default:
			builder.WriteRune('-')
		}
	}
	builder.WriteString("]")
	return builder.String()
}

func drawCandle(c data.Candle, width int) string {
	if width < 6 {
		width = 6
	}
	if c.High <= c.Low {
		return "[" + strings.Repeat("·", width) + "]"
	}
	rangeWidth := c.High - c.Low
	ratioClose := (c.Close - c.Low) / rangeWidth
	ratioEntry := (c.Entry - c.Low) / rangeWidth

	ratioClose = math.Max(0, math.Min(1, ratioClose))
	ratioEntry = math.Max(0, math.Min(1, ratioEntry))

	closePos := int(math.Round(ratioClose * float64(width-1)))
	entryPos := int(math.Round(ratioEntry * float64(width-1)))

	var builder strings.Builder
	builder.Grow(width + 2)
	builder.WriteString("[")
	for i := 0; i < width; i++ {
		switch i {
		case closePos:
			builder.WriteString("[green]|[-]")
		case entryPos:
			builder.WriteString("[cyan]|[-]")
		default:
			builder.WriteString("·")
		}
	}
	builder.WriteString("]")
	return builder.String()
}

func writeKeyboardBanner(screen tcell.Screen) {
	if screen == nil {
		return
	}
	style := tcell.StyleDefault.Foreground(tcell.ColorYellow).Background(tcell.ColorBlack)
	message := "Keyboard Shortcuts"
	for i, r := range message {
		screen.SetContent(i, 0, r, nil, style)
	}
	screen.Show()
}

func clearKeyboardBanner(screen tcell.Screen) {
	if screen == nil {
		return
	}
	style := tcell.StyleDefault
	message := "Keyboard Shortcuts"
	for i := range message {
		screen.SetContent(i, 0, ' ', nil, style)
	}
	screen.Show()
}
