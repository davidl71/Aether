package main

import (
	"fmt"
	"os"
	"strings"
	"testing"
	"time"

	pbv1 "github.com/dlowes/ib-platform/agents/go/proto/v1"
	"google.golang.org/protobuf/proto"
)

func TestEnv_Fallback(t *testing.T) {
	got := env("UNLIKELY_ENV_VAR_88888", "fallback")
	if got != "fallback" {
		t.Fatalf("expected fallback, got %q", got)
	}
}

func TestILPLineFormat(t *testing.T) {
	tk := tick{
		Symbol: "SPY",
		Bid:    450.10,
		Ask:    450.20,
		Last:   450.15,
		Volume: 12345,
	}
	ts := time.Date(2025, 1, 15, 10, 30, 0, 0, time.UTC)

	sym := strings.ReplaceAll(tk.Symbol, " ", "\\ ")
	line := fmt.Sprintf("market_data,symbol=%s bid=%.6f,ask=%.6f,last=%.6f,volume=%di %d\n",
		sym, tk.Bid, tk.Ask, tk.Last, tk.Volume, ts.UnixNano())

	if !strings.HasPrefix(line, "market_data,symbol=SPY ") {
		t.Fatalf("bad prefix: %s", line)
	}
	if !strings.Contains(line, "bid=450.100000") {
		t.Fatalf("missing bid: %s", line)
	}
	if !strings.Contains(line, "ask=450.200000") {
		t.Fatalf("missing ask: %s", line)
	}
	if !strings.Contains(line, "volume=12345i") {
		t.Fatalf("missing volume: %s", line)
	}
	if !strings.HasSuffix(line, "\n") {
		t.Fatal("line must end with newline")
	}
}

func TestILPLineFormat_SpaceInSymbol(t *testing.T) {
	sym := strings.ReplaceAll("BRK B", " ", "\\ ")
	if sym != "BRK\\ B" {
		t.Fatalf("space escaping failed: %q", sym)
	}
}

func TestTickStruct_Defaults(t *testing.T) {
	var tk tick
	if tk.Symbol != "" {
		t.Fatal("zero value symbol should be empty")
	}
	if tk.Volume != 0 {
		t.Fatal("zero value volume should be 0")
	}
}

func TestEnvBool_True(t *testing.T) {
	for _, v := range []string{"1", "true", "yes", "TRUE", " 1 "} {
		t.Run(v, func(t *testing.T) {
			_ = os.Setenv("TEST_ENV_BOOL", v)
			defer func() { _ = os.Unsetenv("TEST_ENV_BOOL") }()
			if !envBool("TEST_ENV_BOOL") {
				t.Errorf("envBool(%q) should be true", v)
			}
		})
	}
}

func TestEnvBool_False(t *testing.T) {
	_ = os.Unsetenv("TEST_ENV_BOOL_FALSE")
	if envBool("TEST_ENV_BOOL_FALSE") {
		t.Error("unset should be false")
	}
	for _, v := range []string{"0", "false", "no", ""} {
		t.Run(v, func(t *testing.T) {
			_ = os.Setenv("TEST_ENV_BOOL", v)
			defer func() { _ = os.Unsetenv("TEST_ENV_BOOL") }()
			if envBool("TEST_ENV_BOOL") {
				t.Errorf("envBool(%q) should be false", v)
			}
		})
	}
}

func TestParseTickPayload_Flat(t *testing.T) {
	data := []byte(`{"symbol":"SPY","bid":450.1,"ask":450.2,"last":450.15,"volume":100}`)
	tick, ts, err := parseTickPayload(data, "market.data.tick.SPY")
	if err != nil {
		t.Fatalf("parse: %v", err)
	}
	if tick.Symbol != "SPY" {
		t.Errorf("symbol: got %q", tick.Symbol)
	}
	if tick.Bid != 450.1 || tick.Ask != 450.2 || tick.Volume != 100 {
		t.Errorf("fields: bid=%.2f ask=%.2f vol=%d", tick.Bid, tick.Ask, tick.Volume)
	}
	if ts.IsZero() {
		t.Error("time should be set")
	}
}

func TestParseTickPayload_Envelope(t *testing.T) {
	data := []byte(`{"payload":{"symbol":"QQQ","bid":380,"ask":380.05,"last":380.02,"volume":200}}`)
	tick, _, err := parseTickPayload(data, "market.data.tick.QQQ")
	if err != nil {
		t.Fatalf("parse: %v", err)
	}
	if tick.Symbol != "QQQ" {
		t.Errorf("symbol: got %q", tick.Symbol)
	}
	if tick.Bid != 380 || tick.Volume != 200 {
		t.Errorf("bid=%.2f vol=%d", tick.Bid, tick.Volume)
	}
}

func TestParseTickPayload_SymbolFromSubject(t *testing.T) {
	data := []byte(`{"bid":1,"ask":2,"last":1.5,"volume":0}`)
	tick, _, err := parseTickPayload(data, "market.data.tick.IWM")
	if err != nil {
		t.Fatalf("parse: %v", err)
	}
	if tick.Symbol != "IWM" {
		t.Errorf("symbol from subject: got %q", tick.Symbol)
	}
}

func TestParseTickPayload_NoSymbol(t *testing.T) {
	data := []byte(`{"bid":1,"ask":2,"last":1.5,"volume":0}`)
	_, _, err := parseTickPayload(data, "short.subject")
	if err == nil {
		t.Fatal("expected error when no symbol in payload or subject")
	}
}

func TestParseTickPayload_InvalidJSON(t *testing.T) {
	_, _, err := parseTickPayload([]byte(`{not json}`), "market.data.tick.SPY")
	if err == nil {
		t.Fatal("expected error for invalid JSON")
	}
}

func TestDecodeEnvelope_Proto(t *testing.T) {
	mde := &pbv1.MarketDataEvent{
		Symbol: "SPY",
		Bid:    450.10,
		Ask:    450.20,
		Last:   450.15,
		Volume: 9999,
	}
	mdeBytes, err := proto.Marshal(mde)
	if err != nil {
		t.Fatalf("marshal MarketDataEvent: %v", err)
	}
	env := &pbv1.NatsEnvelope{
		MessageType: "MarketDataEvent",
		Payload:     mdeBytes,
	}
	envBytes, err := proto.Marshal(env)
	if err != nil {
		t.Fatalf("marshal NatsEnvelope: %v", err)
	}

	tk, ts, err := decodeEnvelope(envBytes, "market.data.SPY")
	if err != nil {
		t.Fatalf("decode: %v", err)
	}
	if tk.Symbol != "SPY" {
		t.Errorf("symbol: got %q", tk.Symbol)
	}
	if tk.Bid != 450.10 || tk.Ask != 450.20 || tk.Volume != 9999 {
		t.Errorf("fields: bid=%.2f ask=%.2f vol=%d", tk.Bid, tk.Ask, tk.Volume)
	}
	if ts.IsZero() {
		t.Error("timestamp should not be zero")
	}
}

func TestDecodeEnvelope_ProtoSymbolFromSubject(t *testing.T) {
	mde := &pbv1.MarketDataEvent{Bid: 1, Ask: 2, Last: 1.5} // no symbol
	mdeBytes, _ := proto.Marshal(mde)
	env := &pbv1.NatsEnvelope{MessageType: "MarketDataEvent", Payload: mdeBytes}
	envBytes, _ := proto.Marshal(env)

	tk, _, err := decodeEnvelope(envBytes, "market.data.tick.IWM")
	if err != nil {
		t.Fatalf("decode: %v", err)
	}
	if tk.Symbol != "IWM" {
		t.Errorf("symbol from subject: got %q", tk.Symbol)
	}
}

func TestDecodeEnvelope_JSONFallback(t *testing.T) {
	data := []byte(`{"symbol":"QQQ","bid":380,"ask":380.05,"last":380.02,"volume":200}`)
	tk, _, err := decodeEnvelope(data, "market.data.QQQ")
	if err != nil {
		t.Fatalf("json fallback: %v", err)
	}
	if tk.Symbol != "QQQ" || tk.Bid != 380 || tk.Volume != 200 {
		t.Errorf("fields: sym=%s bid=%.2f vol=%d", tk.Symbol, tk.Bid, tk.Volume)
	}
}
