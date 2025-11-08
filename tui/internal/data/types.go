package data

import "time"

// Candle represents OHLC data for a period.
type Candle struct {
	Open    float64   `json:"open"`
	High    float64   `json:"high"`
	Low     float64   `json:"low"`
	Close   float64   `json:"close"`
	Volume  float64   `json:"volume"`
	Entry   float64   `json:"entry"`
	Updated time.Time `json:"updated"`
}

// OptionStrike represents call/put quotes for a given strike.
type OptionStrike struct {
	Strike  float64 `json:"strike"`
	CallBid float64 `json:"call_bid"`
	CallAsk float64 `json:"call_ask"`
	PutBid  float64 `json:"put_bid"`
	PutAsk  float64 `json:"put_ask"`
}

// OptionSeries is a set of strikes for a particular expiration.
type OptionSeries struct {
	Expiration string         `json:"expiration"`
	Strikes    []OptionStrike `json:"strikes"`
}

// SymbolSnapshot describes top-line data for an underlying or combo.
type SymbolSnapshot struct {
	Symbol       string         `json:"symbol"`
	Last         float64        `json:"last"`
	Bid          float64        `json:"bid"`
	Ask          float64        `json:"ask"`
	Spread       float64        `json:"spread"`
	ROI          float64        `json:"roi"`
	MakerCount   int            `json:"maker_count"`
	TakerCount   int            `json:"taker_count"`
	Volume       int            `json:"volume"`
	Candle       Candle         `json:"candle"`
	OptionChains []OptionSeries `json:"option_chains"`
}

// Position snapshot.
type Position struct {
	Name           string  `json:"name"`
	Quantity       int     `json:"quantity"`
	ROI            float64 `json:"roi"`
	MakerCount     int     `json:"maker_count"`
	TakerCount     int     `json:"taker_count"`
	RebateEstimate float64 `json:"rebate_estimate"`
	Vega           float64 `json:"vega"`
	Theta          float64 `json:"theta"`
	FairDiff       float64 `json:"fair_diff"`
	Candle         Candle  `json:"candle"`
}

// Order event.
type Order struct {
	Timestamp time.Time `json:"timestamp"`
	Text      string    `json:"text"`
	Severity  string    `json:"severity"`
}

// Alert event.
type Alert struct {
	Timestamp time.Time `json:"timestamp"`
	Text      string    `json:"text"`
	Severity  string    `json:"severity"`
}

// AccountMetrics summarises account state.
type AccountMetrics struct {
	NetLiq            float64 `json:"net_liq"`
	BuyingPower       float64 `json:"buying_power"`
	ExcessLiquidity   float64 `json:"excess_liquidity"`
	MarginRequirement float64 `json:"margin_requirement"`
	Commissions       float64 `json:"commissions"`
	PortalOK          bool    `json:"portal_ok"`
	TWSOK             bool    `json:"tws_ok"`
	ORATSOK           bool    `json:"orats_ok"`
	QuestDBOK         bool    `json:"questdb_ok"`
}

// Snapshot is the aggregate data served to the TUI.
type Snapshot struct {
	GeneratedAt time.Time        `json:"generated_at"`
	Mode        string           `json:"mode"`
	Strategy    string           `json:"strategy"`
	AccountID   string           `json:"account_id"`
	Metrics     AccountMetrics   `json:"metrics"`
	Symbols     []SymbolSnapshot `json:"symbols"`
	Positions   []Position       `json:"positions"`
	Historic    []Position       `json:"historic"`
	Orders      []Order          `json:"orders"`
	Alerts      []Alert          `json:"alerts"`
}

// Provider produces snapshots for the TUI.
type Provider interface {
	Snapshots() <-chan Snapshot
	Stop()
}
