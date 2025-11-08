package app

import (
	"context"
	"fmt"
	"math"
	"os"
	"sort"
	"strings"
	"time"

	"github.com/davidlowes/ib_box_spread_full_universal/tui/internal/data"

	"github.com/gdamore/tcell/v2"
	"github.com/rivo/tview"
)

type uiState struct {
	latest         data.Snapshot
	symbolCache    map[string]data.SymbolSnapshot
	watchlist      []string
	windowWidth    int
	windowHeight   int
	dashboardTable *tview.Table
}

type runOptions struct {
	screen   tcell.Screen
	provider data.Provider
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

var mainTabDefinitions = []tabDefinition{
	{id: "dashboard", title: "Dashboard"},
	{id: "current", title: "Current Positions"},
	{id: "historic", title: "Historic Positions"},
	{id: "orders", title: "Orders"},
	{id: "alerts", title: "Alerts"},
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
		if endpoint := os.Getenv("TUI_API_URL"); endpoint != "" {
			rest := data.NewRestProvider(endpoint, 2*time.Second)
			rest.Start(ctx)
			provider = rest
		} else {
			mock := data.NewMockProvider()
			mock.Start(ctx, time.Second)
			provider = mock
		}
	}
	if provider == nil {
		return fmt.Errorf("no data provider available")
	}
	defer provider.Stop()

	header := buildHeader()
	dashboard := buildDashboard()
	positions := buildPositions()
	historic := buildHistoric()
	orders := buildOrders()
	alerts := buildAlerts()
	controls := buildControls()
	updateControls(controls, state)

	tabs := buildTabs(mainTabDefinitions)

	tabContent := map[string]tview.Primitive{
		"dashboard": dashboard,
		"current":   positions,
		"historic":  historic,
		"orders":    orders,
		"alerts":    alerts,
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
		case "current":
			app.SetFocus(positions)
		case "historic":
			app.SetFocus(historic)
		case "orders":
			app.SetFocus(orders)
		case "alerts":
			app.SetFocus(alerts)
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

	dashboardTable := extractDashboardTable(dashboard)
	if dashboardTable != nil {
		dashboardTable.SetSelectable(true, false)
		dashboardTable.SetFixed(1, 0)
	}
	state.dashboardTable = dashboardTable

	positions.SetSelectable(true, false)
	positions.SetFixed(1, 0)

	historic.SetSelectable(true, false)
	historic.SetFixed(1, 0)

	if dashboardTable != nil {
		dashboardTable.SetSelectedFunc(func(row, column int) {
			if row <= 0 {
				return
			}
			index := row - 1
			if index >= len(state.watchlist) {
				return
			}
			symbol := state.watchlist[index]
			showOptionChain(app, layout, state, symbol)
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
	if state.dashboardTable != nil {
		app.SetFocus(state.dashboardTable)
	}

	app.SetBeforeDrawFunc(func(screen tcell.Screen) bool {
		width, height := screen.Size()
		if state.setWindowSize(width, height) && !state.latest.GeneratedAt.IsZero() {
			app.QueueUpdateDraw(func() {
				updateControls(controls, state)
				updateDashboard(state.dashboardTable, state)
				updatePositions(positions, state.latest.Positions, "Current Positions", state.sparklineWidth())
				updatePositions(historic, state.latest.Historic, "Historic Positions", state.sparklineWidth())
			})
		}
		return false
	})

	layout.SetInputCapture(func(event *tcell.EventKey) *tcell.EventKey {
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

	go func() {
		for snap := range provider.Snapshots() {
			snapshot := snap
			app.QueueUpdateDraw(func() {
				state.observeSnapshot(snapshot)
				updateHeader(header, snapshot)
				updateDashboard(state.dashboardTable, state)
				updatePositions(positions, snapshot.Positions, "Current Positions", state.sparklineWidth())
				updatePositions(historic, snapshot.Historic, "Historic Positions", state.sparklineWidth())
				updateOrders(orders, snapshot.Orders)
				updateAlerts(alerts, snapshot.Alerts)
				updateControls(controls, state)
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
	table.SetCell(0, 0, tview.NewTableCell("[cyan]IB Box Spread Terminal[-]"))
	table.SetCell(0, 1, tview.NewTableCell("Time: [white]--:--:--[-]").SetAlign(tview.AlignRight))
	table.SetCell(1, 0, tview.NewTableCell("Mode: [green]DRY-RUN[-]   Strategy: [green]RUNNING[-]   Account: [cyan]DU123456[-]"))
	table.SetCell(2, 0, tview.NewTableCell("TWS: [green]OK[-]   ORATS: [green]ENABLED[-]   Portal: [green]OK[-]   QuestDB: [green]OK[-]"))
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
	table := tview.NewTable().SetBorders(false)
	table.SetBorder(true).SetTitle("Symbols")
	layout.AddItem(table, 0, 1, true)
	return layout
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

func showModal(app *tview.Application, root tview.Primitive, message string) {
	modal := tview.NewModal().
		SetText(fmt.Sprintf("%s\nPress Enter to continue", message)).
		AddButtons([]string{"OK"})
	modal.SetDoneFunc(func(buttonIndex int, buttonLabel string) {
		app.SetRoot(root, true)
	})
	app.SetRoot(modal, true)
}

func updateHeader(table *tview.Table, snap data.Snapshot) {
	table.Clear()
	title := fmt.Sprintf("[cyan]IB Box Spread Terminal[-]    Time: [white]%s[-]", snap.GeneratedAt.Format("15:04:05"))
	table.SetCell(0, 0, tview.NewTableCell(title).SetExpansion(1))
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

	for row, symbol := range state.watchlist {
		dataPoint, _ := state.symbolData(symbol)
		r := row + 1
		for colIndex, col := range columns {
			table.SetCell(r, colIndex, tview.NewTableCell(col.render(dataPoint)).SetAlign(col.align))
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
		fmt.Fprintf(text, "%s %s%s[-]\n", alert.Timestamp.Format("15:04:05"), severityColor(alert.Severity), alert.Text)
	}
}

func updateControls(view *tview.TextView, state *uiState) {
	if view == nil {
		return
	}

	width := state.windowWidth
	var text string
	switch {
	case width >= 130:
		text = "Controls: [cyan][Tab/Shift+Tab][-] tabs  [green][Enter][-] option chain  [magenta][A or +][-] add symbol  [yellow][?][-] help  [blue][B][-] buy mock  [blue][Shift+S][-] sell mock  [red][Q][-] quit"
	case width >= 100:
		text = "Controls: [cyan]Tab[-]/[cyan]Shift+Tab[-] tabs  [green]Enter[-] chain  [magenta]A[-] add  [yellow]?[-] help  [red]Q[-] quit"
	default:
		text = "Controls: Tab • Enter • A • ? • Q"
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
	help := tview.NewTextView().
		SetDynamicColors(true).
		SetText("[yellow]Keyboard Shortcuts[-]\n\n?  Show this help\nA/+  Add symbol to dashboard\nEnter  Open option chain for selection\nTab / Shift+Tab  Cycle tabs\nB  Mock buy combo\nShift+S  Mock sell combo\nQ  Quit application").
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
	if len(sym.OptionChains) == 0 || len(sym.OptionChains[0].Strikes) == 0 {
		showSymbolDetail(app, root, sym)
		return
	}

	series := sym.OptionChains[0]
	strikes := append([]data.OptionStrike(nil), series.Strikes...)
	sort.Slice(strikes, func(i, j int) bool {
		return strikes[i].Strike < strikes[j].Strike
	})
	if len(strikes) == 0 {
		showSymbolDetail(app, root, sym)
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

	table := tview.NewTable().SetBorders(false)
	table.SetBorder(true).SetTitle(fmt.Sprintf("%s %s Option Chain", sym.Symbol, series.Expiration))

	headers := []string{"Strike", "Call Bid", "Call Ask", "Put Bid", "Put Ask"}
	for col, header := range headers {
		table.SetCell(0, col, tview.NewTableCell("[yellow]"+header+"[-]").SetAlign(tview.AlignCenter))
	}

	for row, strike := range visible {
		r := row + 1
		table.SetCell(r, 0, tview.NewTableCell(fmt.Sprintf("%.2f", strike.Strike)).SetAlign(tview.AlignRight))
		table.SetCell(r, 1, tview.NewTableCell(fmt.Sprintf("%.2f", strike.CallBid)).SetAlign(tview.AlignRight))
		table.SetCell(r, 2, tview.NewTableCell(fmt.Sprintf("%.2f", strike.CallAsk)).SetAlign(tview.AlignRight))
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

	summary := tview.NewTextView().
		SetDynamicColors(true).
		SetText(fmt.Sprintf("[white]%s[-] Last %.2f  Bid %.2f  Ask %.2f  Spread %.2f  ROI %.2f%%", sym.Symbol, sym.Last, sym.Bid, sym.Ask, sym.Spread, sym.ROI)).
		SetWrap(true)

	footer := tview.NewTextView().
		SetDynamicColors(true).
		SetText("[gray]Use arrows to browse. Esc or Q to return.[-]").
		SetTextAlign(tview.AlignCenter)

	container := tview.NewFlex().SetDirection(tview.FlexRow).
		AddItem(summary, 2, 0, false).
		AddItem(table, 0, 1, true).
		AddItem(footer, 1, 0, false)

	close := func() {
		app.SetRoot(root, true)
		if state.dashboardTable != nil {
			app.SetFocus(state.dashboardTable)
		}
	}

	table.SetDoneFunc(func(key tcell.Key) {
		close()
	})
	table.SetInputCapture(func(event *tcell.EventKey) *tcell.EventKey {
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
		switch {
		case i == closePos:
			builder.WriteString("[green]|[-]")
		case i == entryPos:
			builder.WriteString("[cyan]|[-]")
		default:
			builder.WriteString("·")
		}
	}
	builder.WriteString("]")
	return builder.String()
}
