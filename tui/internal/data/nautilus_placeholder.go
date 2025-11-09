package data

import (
	"context"
	"log"
	"time"
)

type NautilusPlaceholderProvider struct {
	mock *MockProvider
}

func NewNautilusPlaceholderProvider(ctx context.Context, interval time.Duration) *NautilusPlaceholderProvider {
	mock := NewMockProvider()
	mock.Start(ctx, interval)
	log.Println("[tui] Nautilus integration placeholder active; returning mock data")
	return &NautilusPlaceholderProvider{mock: mock}
}

func (p *NautilusPlaceholderProvider) Snapshots() <-chan Snapshot {
	return p.mock.Snapshots()
}

func (p *NautilusPlaceholderProvider) Stop() {
	if p.mock != nil {
		p.mock.Stop()
	}
}

func (p *NautilusPlaceholderProvider) AddSymbol(symbol string) error {
	if p.mock == nil {
		return nil
	}
	return p.mock.AddSymbol(symbol)
}
