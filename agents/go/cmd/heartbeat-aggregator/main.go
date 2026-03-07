// heartbeat-aggregator listens for service heartbeats on NATS and
// exposes a /health endpoint that returns aggregate service status.
//
// Services publish JSON heartbeats to "heartbeat.<service>" every few
// seconds (or NatsEnvelope protobuf with source set).  This aggregator
// tracks the last-seen time for each and
// marks services as "down" after a configurable timeout.
//
// Environment:
//
//	NATS_URL          (default "nats://localhost:4222")
//	LISTEN_ADDR       (default ":8090")
//	STALE_TIMEOUT     (default "15s")
package main

import (
	"context"
	"encoding/json"
	"log/slog"
	"net/http"
	"os"
	"os/signal"
	"sync"
	"strings"
	"syscall"
	"time"

	"github.com/nats-io/nats.go"
	"google.golang.org/protobuf/proto"

	pbv1 "github.com/dlowes/ib-platform/agents/go/proto/v1"
)

func env(key, fallback string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return fallback
}

type serviceState struct {
	mu       sync.RWMutex
	services map[string]time.Time
	stale    time.Duration
}

func (s *serviceState) update(name string) {
	s.mu.Lock()
	s.services[name] = time.Now()
	s.mu.Unlock()
}

type serviceStatus struct {
	Name     string `json:"name"`
	Status   string `json:"status"`
	LastSeen string `json:"last_seen"`
}

func (s *serviceState) snapshot() []serviceStatus {
	s.mu.RLock()
	defer s.mu.RUnlock()
	now := time.Now()
	out := make([]serviceStatus, 0, len(s.services))
	for name, ts := range s.services {
		status := "ok"
		if now.Sub(ts) > s.stale {
			status = "down"
		}
		out = append(out, serviceStatus{
			Name:     name,
			Status:   status,
			LastSeen: ts.Format(time.RFC3339),
		})
	}
	return out
}

// serviceNameFromMessage returns the service name from msg: if data is a
// NatsEnvelope protobuf with source set, use it; otherwise use subject
// "heartbeat.<name>". (P2-B / T-1772887221969976131: decode NatsEnvelope in Go agents.)
func serviceNameFromMessage(data []byte, subject string) string {
	env := &pbv1.NatsEnvelope{}
	if err := proto.Unmarshal(data, env); err == nil {
		if src := strings.TrimSpace(env.GetSource()); src != "" {
			return src
		}
	}
	const prefix = "heartbeat."
	if len(subject) > len(prefix) {
		return subject[len(prefix):]
	}
	return ""
}

func main() {
	slog.SetDefault(slog.New(slog.NewJSONHandler(os.Stdout, nil)))
	natsURL := env("NATS_URL", nats.DefaultURL)
	listenAddr := env("LISTEN_ADDR", ":8090")
	staleStr := env("STALE_TIMEOUT", "15s")

	staleDur, err := time.ParseDuration(staleStr)
	if err != nil {
		slog.Error("bad STALE_TIMEOUT", "error", err)
		os.Exit(1)
	}

	state := &serviceState{
		services: make(map[string]time.Time),
		stale:    staleDur,
	}

	nc, err := nats.Connect(natsURL,
		nats.RetryOnFailedConnect(true),
		nats.MaxReconnects(-1),
	)
	if err != nil {
		slog.Error("nats connect", "error", err)
		os.Exit(1)
	}
	defer nc.Close()

	sub, err := nc.Subscribe("heartbeat.>", func(msg *nats.Msg) {
		svc := serviceNameFromMessage(msg.Data, msg.Subject)
		if svc != "" {
			state.update(svc)
		}
	})
	if err != nil {
		slog.Error("subscribe", "error", err)
		os.Exit(1)
	}
	defer func() { _ = sub.Unsubscribe() }()

	mux := http.NewServeMux()
	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(map[string]any{
			"services": state.snapshot(),
			"as_of":    time.Now().Format(time.RFC3339),
		})
	})

	srv := &http.Server{Addr: listenAddr, Handler: mux}

	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()

	go func() {
		slog.Info("heartbeat-aggregator listening", "addr", listenAddr)
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			slog.Error("server error", "error", err)
			os.Exit(1)
		}
	}()

	<-ctx.Done()
	shutCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	_ = srv.Shutdown(shutCtx)
	slog.Info("heartbeat-aggregator stopped")
}
