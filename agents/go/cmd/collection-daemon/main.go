// collection-daemon is the unified thin Go collection daemon (Epic E5).
// It subscribes to NATS for C++ events (market data, signals, decisions),
// writes live state to NATS KV (P2-C), and will eventually write to QuestDB.
// This slice implements: NATS subscribe + NatsEnvelope decode + NATS KV write + stub/QuestDB later + /metrics.
//
// Environment:
//
//	NATS_URL        (default "nats://localhost:4222")
//	NATS_SUBJECTS   comma-separated subjects (default "market-data.tick.>,strategy.signal.>,strategy.decision.>")
//	NATS_KV_BUCKET  JetStream KV bucket for live state (default "LIVE_STATE"); empty disables KV write
//	METRICS_LISTEN  (default ":9090")
package main

import (
	"context"
	"fmt"
	"log/slog"
	"net/http"
	"os"
	"os/signal"
	"strings"
	"sync"
	"sync/atomic"
	"syscall"
	"time"

	"github.com/nats-io/nats.go"
	pbv1 "github.com/dlowes/ib-platform/agents/go/proto/v1"
	"google.golang.org/protobuf/proto"
)

func env(key, fallback string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return fallback
}

// metrics holds Prometheus-style counters (atomic for lock-free reads).
type metrics struct {
	messagesReceived atomic.Uint64
	decodeErrors     atomic.Uint64
	writeErrors      atomic.Uint64
	byType           map[string]*atomic.Uint64
	mu               sync.Mutex
}

func newMetrics() *metrics {
	m := &metrics{byType: make(map[string]*atomic.Uint64)}
	for _, t := range []string{"MarketDataEvent", "StrategySignal", "StrategyDecision", "unknown"} {
		m.byType[t] = &atomic.Uint64{}
	}
	return m
}

func (m *metrics) incReceived(messageType string) {
	m.messagesReceived.Add(1)
	m.mu.Lock()
	defer m.mu.Unlock()
	if c, ok := m.byType[messageType]; ok {
		c.Add(1)
	} else {
		m.byType["unknown"].Add(1)
	}
}

func (m *metrics) incDecodeError() { m.decodeErrors.Add(1) }
func (m *metrics) incWriteError()  { m.writeErrors.Add(1) }

// stubWriter logs the event (Phase 0). Later: write to QuestDB ILP. KV write is done in handleMsg when kv is set.
func stubWriter(logger *slog.Logger, subject, messageType string, payload []byte) {
	logger.Debug("stub write", "subject", subject, "message_type", messageType, "payload_len", len(payload))
}

// kvKey returns a stable key for NATS KV from subject and message type (e.g. "market_data.SPY", "strategy_decision.AAPL").
func kvKey(subject, messageType string) string {
	parts := strings.Split(subject, ".")
	sym := ""
	if len(parts) > 0 {
		sym = parts[len(parts)-1]
	}
	if sym == "" {
		sym = "default"
	}
	return messageType + "." + sym
}

func main() {
	slog.SetDefault(slog.New(slog.NewJSONHandler(os.Stdout, nil)))
	natsURL := env("NATS_URL", nats.DefaultURL)
	metricsListen := env("METRICS_LISTEN", ":9090")
	subjectsStr := env("NATS_SUBJECTS", "market-data.tick.>,strategy.signal.>,strategy.decision.>")
	subjects := strings.Split(strings.ReplaceAll(subjectsStr, " ", ""), ",")

	m := newMetrics()

	nc, err := nats.Connect(natsURL,
		nats.RetryOnFailedConnect(true),
		nats.MaxReconnects(-1),
		nats.ReconnectWait(2*time.Second),
	)
	if err != nil {
		slog.Error("nats connect", "error", err)
		os.Exit(1)
	}
	defer nc.Close()

	var kv nats.KeyValue
	if bucket := strings.TrimSpace(env("NATS_KV_BUCKET", "LIVE_STATE")); bucket != "" {
		js, err := nc.JetStream()
		if err != nil {
			slog.Error("jetstream", "error", err)
			os.Exit(1)
		}
		kv, err = js.CreateKeyValue(&nats.KeyValueConfig{Bucket: bucket})
		if err != nil {
			slog.Error("create kv bucket", "bucket", bucket, "error", err)
			os.Exit(1)
		}
		slog.Info("kv bucket ready", "bucket", bucket)
	}

	handleMsg := func(data []byte, subj string) {
		env := &pbv1.NatsEnvelope{}
		if err := proto.Unmarshal(data, env); err != nil {
			m.incDecodeError()
			slog.Debug("decode envelope failed, skipping", "error", err, "subject", subj)
			return
		}
		msgType := env.GetMessageType()
		if msgType == "" {
			msgType = "unknown"
		}
		m.incReceived(msgType)

		stubWriter(slog.Default(), subj, msgType, env.GetPayload())
		if kv != nil {
			key := kvKey(subj, msgType)
			if _, err := kv.Put(key, env.GetPayload()); err != nil {
				m.incWriteError()
				slog.Debug("kv put", "key", key, "error", err)
			}
		}

		switch msgType {
		case "MarketDataEvent":
			inner := &pbv1.MarketDataEvent{}
			if err := proto.Unmarshal(env.GetPayload(), inner); err == nil {
				slog.Debug("market_data", "symbol", inner.GetSymbol(), "bid", inner.GetBid(), "ask", inner.GetAsk())
			}
		case "StrategySignal":
			inner := &pbv1.StrategySignal{}
			if err := proto.Unmarshal(env.GetPayload(), inner); err == nil {
				slog.Debug("strategy_signal", "symbol", inner.GetSymbol(), "price", inner.GetPrice())
			}
		case "StrategyDecision":
			inner := &pbv1.StrategyDecision{}
			if err := proto.Unmarshal(env.GetPayload(), inner); err == nil {
				slog.Debug("strategy_decision", "symbol", inner.GetSymbol(), "side", inner.GetSide(), "quantity", inner.GetQuantity())
			}
		}
	}

	for _, subj := range subjects {
		subj := strings.TrimSpace(subj)
		if subj == "" {
			continue
		}
		sub, err := nc.Subscribe(subj, func(msg *nats.Msg) {
			handleMsg(msg.Data, msg.Subject)
		})
		if err != nil {
			slog.Error("subscribe", "subject", subj, "error", err)
			os.Exit(1)
		}
		defer func(s *nats.Subscription) { _ = s.Unsubscribe() }(sub)
		slog.Info("subscribed", "subject", subj)
	}

	mux := http.NewServeMux()
	mux.HandleFunc("/metrics", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "text/plain; version=0.0.4; charset=utf-8")
		total := m.messagesReceived.Load()
		decodeErr := m.decodeErrors.Load()
		writeErr := m.writeErrors.Load()
		_, _ = w.Write([]byte("# HELP collection_daemon_messages_received_total Total messages received from NATS\n"))
		_, _ = w.Write([]byte("# TYPE collection_daemon_messages_received_total counter\n"))
		_, _ = w.Write([]byte("collection_daemon_messages_received_total " + fmt.Sprint(total) + "\n"))
		_, _ = w.Write([]byte("# HELP collection_daemon_decode_errors_total Decode errors\n"))
		_, _ = w.Write([]byte("# TYPE collection_daemon_decode_errors_total counter\n"))
		_, _ = w.Write([]byte("collection_daemon_decode_errors_total " + fmt.Sprint(decodeErr) + "\n"))
		_, _ = w.Write([]byte("# HELP collection_daemon_write_errors_total Write errors\n"))
		_, _ = w.Write([]byte("# TYPE collection_daemon_write_errors_total counter\n"))
		_, _ = w.Write([]byte("collection_daemon_write_errors_total " + fmt.Sprint(writeErr) + "\n"))
	})

	srv := &http.Server{Addr: metricsListen, Handler: mux}
	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()

	go func() {
		slog.Info("metrics listening", "addr", metricsListen)
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			slog.Error("metrics server", "error", err)
			os.Exit(1)
		}
	}()

	slog.Info("collection-daemon running (NATS subscribe + stub writer), ctrl-c to stop")
	<-ctx.Done()
	shutCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	_ = srv.Shutdown(shutCtx)
	slog.Info("collection-daemon stopped")
}
