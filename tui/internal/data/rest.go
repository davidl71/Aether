package data

import (
    "context"
    "encoding/json"
    "fmt"
    "net/http"
    "sync"
    "time"
)

// RestProvider polls a REST endpoint for snapshots.
type RestProvider struct {
    client   *http.Client
    endpoint string
    interval time.Duration

    ch     chan Snapshot
    stopCh chan struct{}
    once   sync.Once
}

// NewRestProvider creates a provider that hits the given endpoint.
func NewRestProvider(endpoint string, interval time.Duration) *RestProvider {
    return &RestProvider{
        client: &http.Client{
            Timeout: 10 * time.Second,
        },
        endpoint: endpoint,
        interval: interval,
        ch:       make(chan Snapshot, 1),
        stopCh:   make(chan struct{}),
    }
}

// Start begins polling.
func (p *RestProvider) Start(ctx context.Context) {
    ticker := time.NewTicker(p.interval)
    go func() {
        defer ticker.Stop()
        if snap, err := p.fetch(ctx); err == nil {
            select {
            case p.ch <- snap:
            default:
            }
        }
        for {
            select {
            case <-ctx.Done():
                p.Stop()
                return
            case <-p.stopCh:
                close(p.ch)
                return
            case <-ticker.C:
                if snap, err := p.fetch(ctx); err == nil {
                    select {
                    case p.ch <- snap:
                    default:
                        <-p.ch
                        p.ch <- snap
                    }
                }
            }
        }
    }()
}

func (p *RestProvider) fetch(ctx context.Context) (Snapshot, error) {
    req, err := http.NewRequestWithContext(ctx, http.MethodGet, p.endpoint, nil)
    if err != nil {
        return Snapshot{}, err
    }
    resp, err := p.client.Do(req)
    if err != nil {
        return Snapshot{}, err
    }
    defer resp.Body.Close()

    if resp.StatusCode != http.StatusOK {
        return Snapshot{}, fmt.Errorf("unexpected status: %d", resp.StatusCode)
    }

    var snap Snapshot
    if err := json.NewDecoder(resp.Body).Decode(&snap); err != nil {
        return Snapshot{}, err
    }
    return snap, nil
}

// Snapshots returns snapshot channel.
func (p *RestProvider) Snapshots() <-chan Snapshot {
    return p.ch
}

// Stop stops polling.
func (p *RestProvider) Stop() {
    p.once.Do(func() { close(p.stopCh) })
}

