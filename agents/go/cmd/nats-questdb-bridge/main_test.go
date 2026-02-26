package main

import (
	"fmt"
	"strings"
	"testing"
	"time"
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
