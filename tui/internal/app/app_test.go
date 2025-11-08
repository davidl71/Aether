package app

import (
	"testing"

	"github.com/davidlowes/ib_box_spread_full_universal/tui/internal/data"
)

func TestHealthColor(t *testing.T) {
	if got := healthColor(true, "OK"); got != "[green]OK[-]" {
		t.Fatalf("unexpected healthy color: %s", got)
	}
	if got := healthColor(false, "FAIL"); got != "[red]FAIL[-]" {
		t.Fatalf("unexpected unhealthy color: %s", got)
	}
}

func TestSeverityColor(t *testing.T) {
	tests := map[string]string{
		"warn":     "[yellow]",
		"warning":  "[yellow]",
		"error":    "[red]",
		"critical": "[red]",
		"success":  "[green]",
		"info":     "[blue]",
	}
	for input, want := range tests {
		if got := severityColor(input); got != want {
			t.Fatalf("severity %s -> %s, want %s", input, got, want)
		}
	}
}

func TestDrawCandle(t *testing.T) {
	c := data.Candle{
		Low:   150,
		High:  170,
		Close: 160,
		Entry: 155,
	}
	got := drawCandle(c, 20)
	if len(got) == 0 {
		t.Fatalf("expected non-empty candle representation")
	}
	if got[0] != '[' || got[len(got)-1] != ']' {
		t.Fatalf("expected candle enclosed in brackets, got %s", got)
	}
}
