// collection-daemon is the unified thin Go collection daemon (Epic E5).
// It subscribes to NATS for C++ events (market data, signals, decisions),
// decodes NatsEnvelope records once, and writes them through configured sinks.
// Today those sinks are NATS KV plus structured logging. JetStream durable
// consumption is optional and acks only after all sinks succeed.
//
// Environment:
//
//	NATS_URL        (default "nats://localhost:4222")
//	NATS_USE_JETSTREAM  if "1" or "true", use JetStream durable consumers instead of Core NATS
//	NATS_SUBJECTS   comma-separated subjects (default "market-data.tick.>,strategy.signal.>,strategy.decision.>")
//	NATS_STREAM     JetStream stream name (default "PLATFORM_EVENTS")
//	NATS_DURABLE_PREFIX  durable consumer prefix in JetStream mode (default "collection-daemon")
//	NATS_KV_BUCKET  JetStream KV bucket for live state (default "LIVE_STATE"); empty disables KV write
//	METRICS_LISTEN  (default ":9090")
package main

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"log/slog"
	"net/http"
	"os"
	"os/signal"
	"regexp"
	"strings"
	"sync"
	"sync/atomic"
	"syscall"
	"time"

	pbv1 "github.com/dlowes/ib-platform/agents/go/proto/v1"
	"github.com/nats-io/nats.go"
	"google.golang.org/protobuf/proto"
)

func env(key, fallback string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return fallback
}

func envBool(key string) bool {
	v := strings.ToLower(strings.TrimSpace(os.Getenv(key)))
	return v == "1" || v == "true" || v == "yes"
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

var (
	errDecodeMessage = errors.New("decode message")
	errSinkWrite     = errors.New("sink write")
)

type collectedMessage struct {
	Subject     string
	EnvelopeRaw []byte
	Envelope    *pbv1.NatsEnvelope
	MessageType string
}

type messageSink interface {
	Write(*collectedMessage) error
	Name() string
}

type logSink struct {
	logger *slog.Logger
}

func (s logSink) Name() string { return "log" }

func (s logSink) Write(msg *collectedMessage) error {
	s.logger.Debug(
		"stub write",
		"subject", msg.Subject,
		"message_type", msg.MessageType,
		"payload_len", len(msg.Envelope.GetPayload()),
	)
	return nil
}

type kvSink struct {
	kv nats.KeyValue
}

func (s kvSink) Name() string { return "nats-kv" }

func (s kvSink) Write(msg *collectedMessage) error {
	key := kvKey(msg.Subject, msg.MessageType)
	_, err := s.kv.Put(key, msg.EnvelopeRaw)
	return err
}

func writeToSinks(msg *collectedMessage, sinks []messageSink, m *metrics) error {
	var firstErr error
	for _, sink := range sinks {
		if err := sink.Write(msg); err != nil {
			m.incWriteError()
			slog.Debug("sink write failed", "sink", sink.Name(), "subject", msg.Subject, "error", err)
			if firstErr == nil {
				firstErr = fmt.Errorf("%w: %v", errSinkWrite, err)
			}
			continue
		}
		if sink.Name() == "nats-kv" {
			slog.Debug("kv put", "key", kvKey(msg.Subject, msg.MessageType), "envelope", envelopeSummary(msg.EnvelopeRaw))
		}
	}
	return firstErr
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

func envelopeSummary(data []byte) map[string]any {
	env := &pbv1.NatsEnvelope{}
	if err := proto.Unmarshal(data, env); err != nil {
		return map[string]any{"decode_error": err.Error()}
	}
	payloadHex := ""
	if len(env.GetPayload()) > 0 {
		payloadHex = fmt.Sprintf("%x", env.GetPayload()[:min(len(env.GetPayload()), 16)])
	}
	timestamp := ""
	if ts := env.GetTimestamp(); ts != nil {
		timestamp = ts.AsTime().UTC().Format(time.RFC3339Nano)
	}
	return map[string]any{
		"id":            env.GetId(),
		"source":        env.GetSource(),
		"message_type":  env.GetMessageType(),
		"timestamp":     timestamp,
		"payload_len":   len(env.GetPayload()),
		"payload_hex16": payloadHex,
	}
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}

func decodeCollectedMessage(data []byte, subj string, m *metrics) (*collectedMessage, error) {
	env := &pbv1.NatsEnvelope{}
	if err := proto.Unmarshal(data, env); err != nil {
		m.incDecodeError()
		return nil, err
	}
	msgType := env.GetMessageType()
	if msgType == "" {
		msgType = "unknown"
	}
	m.incReceived(msgType)
	return &collectedMessage{
		Subject:     subj,
		EnvelopeRaw: data,
		Envelope:    env,
		MessageType: msgType,
	}, nil
}

var durableSanitizer = regexp.MustCompile(`[^A-Za-z0-9_-]+`)

func durableName(prefix, subject string) string {
	base := durableSanitizer.ReplaceAllString(subject, "_")
	base = strings.Trim(base, "_")
	if base == "" {
		base = "default"
	}
	return prefix + "-" + base
}

func main() {
	slog.SetDefault(slog.New(slog.NewJSONHandler(os.Stdout, nil)))
	natsURL := env("NATS_URL", nats.DefaultURL)
	useJetStream := envBool("NATS_USE_JETSTREAM")
	streamName := env("NATS_STREAM", "PLATFORM_EVENTS")
	durablePrefix := env("NATS_DURABLE_PREFIX", "collection-daemon")
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

	var js nats.JetStreamContext
	if useJetStream || strings.TrimSpace(env("NATS_KV_BUCKET", "LIVE_STATE")) != "" {
		js, err = nc.JetStream()
		if err != nil {
			slog.Error("jetstream", "error", err)
			os.Exit(1)
		}
	}

	var kv nats.KeyValue
	sinks := []messageSink{logSink{logger: slog.Default()}}
	if bucket := strings.TrimSpace(env("NATS_KV_BUCKET", "LIVE_STATE")); bucket != "" {
		kv, err = js.CreateKeyValue(&nats.KeyValueConfig{Bucket: bucket})
		if err != nil {
			kv, err = js.KeyValue(bucket)
			if err != nil {
				slog.Error("create kv bucket", "bucket", bucket, "error", err)
				os.Exit(1)
			}
		}
		slog.Info("kv bucket ready", "bucket", bucket)
		sinks = append(sinks, kvSink{kv: kv})
	}

	handleMsg := func(data []byte, subj string) error {
		msg, err := decodeCollectedMessage(data, subj, m)
		if err != nil {
			slog.Debug("decode envelope failed, skipping", "error", err, "subject", subj)
			return fmt.Errorf("%w: %v", errDecodeMessage, err)
		}

		if err := writeToSinks(msg, sinks, m); err != nil {
			return err
		}

		switch msg.MessageType {
		case "MarketDataEvent":
			inner := &pbv1.MarketDataEvent{}
			if err := proto.Unmarshal(msg.Envelope.GetPayload(), inner); err == nil {
				slog.Debug("market_data", "symbol", inner.GetSymbol(), "bid", inner.GetBid(), "ask", inner.GetAsk())
			}
		case "StrategySignal":
			inner := &pbv1.StrategySignal{}
			if err := proto.Unmarshal(msg.Envelope.GetPayload(), inner); err == nil {
				slog.Debug("strategy_signal", "symbol", inner.GetSymbol(), "price", inner.GetPrice())
			}
		case "StrategyDecision":
			inner := &pbv1.StrategyDecision{}
			if err := proto.Unmarshal(msg.Envelope.GetPayload(), inner); err == nil {
				slog.Debug("strategy_decision", "symbol", inner.GetSymbol(), "side", inner.GetSide(), "quantity", inner.GetQuantity())
			}
		}
		return nil
	}

	if useJetStream {
		if _, err := js.StreamInfo(streamName); err != nil {
			if _, addErr := js.AddStream(&nats.StreamConfig{
				Name:     streamName,
				Subjects: subjects,
				Storage:  nats.FileStorage,
				MaxAge:   7 * 24 * time.Hour,
			}); addErr != nil {
				slog.Error("ensure stream", "stream", streamName, "error", addErr)
				os.Exit(1)
			}
		}
	}

	for _, rawSubject := range subjects {
		subj := strings.TrimSpace(rawSubject)
		if subj == "" {
			continue
		}
		if useJetStream {
			durable := durableName(durablePrefix, subj)
			sub, err := js.Subscribe(subj, func(msg *nats.Msg) {
				if err := handleMsg(msg.Data, msg.Subject); err != nil {
					slog.Debug("jetstream message processing failed", "subject", msg.Subject, "error", err)
					if errors.Is(err, errDecodeMessage) {
						if termErr := msg.Term(); termErr != nil {
							slog.Debug("jetstream term failed", "subject", msg.Subject, "error", termErr)
						}
						return
					}
					return
				}
				if ackErr := msg.Ack(); ackErr != nil {
					slog.Debug("jetstream ack failed", "subject", msg.Subject, "error", ackErr)
				}
			},
				nats.ManualAck(),
				nats.Durable(durable),
				nats.BindStream(streamName),
			)
			if err != nil {
				slog.Error("jetstream subscribe", "subject", subj, "stream", streamName, "durable", durable, "error", err)
				os.Exit(1)
			}
			defer func(s *nats.Subscription) { _ = s.Unsubscribe() }(sub)
			slog.Info("jetstream subscribed", "subject", subj, "stream", streamName, "durable", durable)
			continue
		}
		sub, err := nc.Subscribe(subj, func(msg *nats.Msg) {
			if err := handleMsg(msg.Data, msg.Subject); err != nil {
				slog.Debug("core message processing failed", "subject", msg.Subject, "error", err)
			}
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
		_, _ = w.Write([]byte("# HELP collection_daemon_messages_by_type_total Messages received by decoded message type\n"))
		_, _ = w.Write([]byte("# TYPE collection_daemon_messages_by_type_total counter\n"))
		m.mu.Lock()
		defer m.mu.Unlock()
		for messageType, counter := range m.byType {
			line := fmt.Sprintf(
				"collection_daemon_messages_by_type_total{message_type=%q} %d\n",
				messageType,
				counter.Load(),
			)
			_, _ = w.Write([]byte(line))
		}
		_, _ = w.Write([]byte("# HELP collection_daemon_config_info Static config for collector wiring\n"))
		_, _ = w.Write([]byte("# TYPE collection_daemon_config_info gauge\n"))
		configInfo, _ := json.Marshal(map[string]any{
			"subjects":      subjects,
			"kv":            kv != nil,
			"jetstream":     useJetStream,
			"stream":        streamName,
			"durablePrefix": durablePrefix,
		})
		_, _ = w.Write([]byte("# collection_daemon_config " + string(configInfo) + "\n"))
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

	slog.Info("collection-daemon running, ctrl-c to stop")
	<-ctx.Done()
	shutCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	_ = srv.Shutdown(shutCtx)
	slog.Info("collection-daemon stopped")
}
