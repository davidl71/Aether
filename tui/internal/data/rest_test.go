package data

import (
    "context"
    "encoding/json"
    "net/http"
    "net/http/httptest"
    "testing"
    "time"
)

func TestRestProvider(t *testing.T) {
    snap := Snapshot{
        Mode:      "LIVE",
        Strategy:  "RUNNING",
        AccountID: "DU123456",
        Metrics: AccountMetrics{
            NetLiq:      100000,
            BuyingPower: 80000,
        },
    }

    server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        _ = json.NewEncoder(w).Encode(snap)
    }))
    defer server.Close()

    ctx, cancel := context.WithCancel(context.Background())
    defer cancel()

    provider := NewRestProvider(server.URL, 50*time.Millisecond)
    provider.Start(ctx)
    defer provider.Stop()

    select {
    case got := <-provider.Snapshots():
        if got.Mode != snap.Mode || got.AccountID != snap.AccountID {
            t.Fatalf("unexpected snapshot: %#v", got)
        }
    case <-time.After(500 * time.Millisecond):
        t.Fatalf("timeout waiting for snapshot")
    }
}

