package main

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"
)

func TestEnv_Fallback(t *testing.T) {
	got := env("UNLIKELY_ENV_VAR_99999", "default")
	if got != "default" {
		t.Fatalf("expected default, got %q", got)
	}
}

func TestServiceState_Update(t *testing.T) {
	s := &serviceState{
		services: make(map[string]time.Time),
		stale:    15 * time.Second,
	}

	s.update("backend")
	s.update("web")

	if len(s.services) != 2 {
		t.Fatalf("expected 2 services, got %d", len(s.services))
	}
}

func TestServiceState_Snapshot_Ok(t *testing.T) {
	s := &serviceState{
		services: make(map[string]time.Time),
		stale:    15 * time.Second,
	}
	s.update("backend")

	snap := s.snapshot()
	if len(snap) != 1 {
		t.Fatalf("expected 1, got %d", len(snap))
	}
	if snap[0].Status != "ok" {
		t.Fatalf("expected ok, got %q", snap[0].Status)
	}
	if snap[0].Name != "backend" {
		t.Fatalf("expected backend, got %q", snap[0].Name)
	}
}

func TestServiceState_Snapshot_Down(t *testing.T) {
	s := &serviceState{
		services: map[string]time.Time{
			"stale-svc": time.Now().Add(-30 * time.Second),
		},
		stale: 15 * time.Second,
	}

	snap := s.snapshot()
	if len(snap) != 1 {
		t.Fatalf("expected 1, got %d", len(snap))
	}
	if snap[0].Status != "down" {
		t.Fatalf("expected down, got %q", snap[0].Status)
	}
}

func TestHealthEndpoint(t *testing.T) {
	s := &serviceState{
		services: make(map[string]time.Time),
		stale:    15 * time.Second,
	}
	s.update("test-svc")

	mux := http.NewServeMux()
	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(map[string]any{
			"services": s.snapshot(),
		})
	})

	req := httptest.NewRequest(http.MethodGet, "/health", nil)
	rec := httptest.NewRecorder()
	mux.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}

	var body map[string]json.RawMessage
	if err := json.NewDecoder(rec.Body).Decode(&body); err != nil {
		t.Fatalf("decode: %v", err)
	}
	if _, ok := body["services"]; !ok {
		t.Fatal("missing services key in response")
	}
}
