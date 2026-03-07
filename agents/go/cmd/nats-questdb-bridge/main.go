// nats-questdb-bridge subscribes to NATS (JetStream or Core) market data
// and writes each tick to QuestDB via ILP.
//
// Environment:
//
//	NATS_URL          (default "nats://localhost:4222")
//	NATS_USE_CORE     if "1" or "true", use Core NATS instead of JetStream
//	NATS_SUBJECT      Core: "market-data.tick.>"  JetStream: "market.data.>"
//	NATS_STREAM       JetStream only (default "MARKET_DATA")
//	QUESTDB_ILP_ADDR  (default "localhost:9009")
package main

import (
	"context"
	"encoding/json"
	"fmt"
	"log/slog"
	"net"
	"os"
	"os/signal"
	"strings"
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

func envBool(key string) bool {
	v := strings.ToLower(strings.TrimSpace(os.Getenv(key)))
	return v == "1" || v == "true" || v == "yes"
}

// DONE(exarp): T-1772887221969976131 — NatsEnvelope protobuf decode implemented in decodeEnvelope.
// DONE(exarp): T-1772887222034956306 — slog used throughout.
type tick struct {
	Symbol    string  `json:"symbol"`
	Bid       float64 `json:"bid"`
	Ask       float64 `json:"ask"`
	Last      float64 `json:"last"`
	Volume    uint64  `json:"volume"`
	Timestamp string  `json:"timestamp"`
}

// parseTickPayload unmarshals message (optional "payload" envelope), fills symbol from subject if needed, parses timestamp.
// Returns (nil, zero, error) on failure.
func parseTickPayload(data []byte, subj string) (*tick, time.Time, error) {
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(data, &raw); err != nil {
		return nil, time.Time{}, err
	}
	var payload []byte
	if p, ok := raw["payload"]; ok {
		payload = p
	} else {
		payload = data
	}
	var t tick
	if err := json.Unmarshal(payload, &t); err != nil {
		return nil, time.Time{}, err
	}
	if t.Symbol == "" {
		parts := strings.Split(subj, ".")
		if len(parts) >= 3 {
			t.Symbol = parts[len(parts)-1]
		}
	}
	if t.Symbol == "" {
		return nil, time.Time{}, fmt.Errorf("no symbol in subject %s or payload", subj)
	}
	ts := time.Now()
	if t.Timestamp != "" {
		if parsed, e := time.Parse(time.RFC3339Nano, t.Timestamp); e == nil {
			ts = parsed
		}
	}
	return &t, ts, nil
}

// decodeEnvelope decodes a NATS message as NatsEnvelope protobuf (primary path) and falls back
// to JSON for legacy or test publishers.  C++ always publishes NatsEnvelope{message_type,payload}.
func decodeEnvelope(data []byte, subj string) (*tick, time.Time, error) {
	env := &pbv1.NatsEnvelope{}
	if err := proto.Unmarshal(data, env); err == nil && env.GetMessageType() == "MarketDataEvent" {
		mde := &pbv1.MarketDataEvent{}
		if err := proto.Unmarshal(env.GetPayload(), mde); err != nil {
			return nil, time.Time{}, fmt.Errorf("decode MarketDataEvent: %w", err)
		}
		sym := mde.GetSymbol()
		if sym == "" {
			parts := strings.Split(subj, ".")
			sym = parts[len(parts)-1]
		}
		if sym == "" {
			return nil, time.Time{}, fmt.Errorf("no symbol in proto or subject %s", subj)
		}
		ts := time.Now()
		if t := env.GetTimestamp(); t != nil {
			ts = t.AsTime()
		}
		return &tick{
			Symbol: sym,
			Bid:    mde.GetBid(),
			Ask:    mde.GetAsk(),
			Last:   mde.GetLast(),
			Volume: mde.GetVolume(),
		}, ts, nil
	}
	// Fallback: JSON (legacy publishers, integration tests).
	return parseTickPayload(data, subj)
}

func main() {
	slog.SetDefault(slog.New(slog.NewJSONHandler(os.Stdout, nil)))
	natsURL := env("NATS_URL", nats.DefaultURL)
	questAddr := env("QUESTDB_ILP_ADDR", "localhost:9009")
	useCore := envBool("NATS_USE_CORE")
	subject := env("NATS_SUBJECT", "market-data.tick.>")
	if !useCore {
		subject = env("NATS_SUBJECT", "market.data.>")
	}

	mode := "JetStream"
	if useCore {
		mode = "Core NATS"
	}
	slog.Info("nats-questdb-bridge starting", "mode", mode, "nats", natsURL, "questdb", questAddr, "subject", subject)

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

	conn, err := net.DialTimeout("tcp", questAddr, 5*time.Second)
	if err != nil {
		slog.Error("questdb connect", "error", err)
		os.Exit(1)
	}
	defer func() { _ = conn.Close() }()

	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()

	writeTick := func(t *tick, ts time.Time) {
		sym := strings.ReplaceAll(t.Symbol, " ", "\\ ")
		line := fmt.Sprintf("market_data,symbol=%s bid=%.6f,ask=%.6f,last=%.6f,volume=%di %d\n",
			sym, t.Bid, t.Ask, t.Last, t.Volume, ts.UnixNano())
		if _, err := conn.Write([]byte(line)); err != nil {
			slog.Error("questdb write", "error", err)
		}
	}

	parseAndWrite := func(data []byte, subj string) bool {
		tick, ts, err := decodeEnvelope(data, subj)
		if err != nil {
			slog.Error("decode message", "error", err)
			return false
		}
		writeTick(tick, ts)
		return true
	}

	if useCore {
		sub, err := nc.Subscribe(subject, func(msg *nats.Msg) {
			parseAndWrite(msg.Data, msg.Subject)
		})
		if err != nil {
			slog.Error("subscribe", "error", err)
			os.Exit(1)
		}
		defer func() { _ = sub.Unsubscribe() }()
	} else {
		streamName := env("NATS_STREAM", "MARKET_DATA")
		js, err := nc.JetStream()
		if err != nil {
			slog.Error("jetstream", "error", err)
			os.Exit(1)
		}
		_, err = js.AddStream(&nats.StreamConfig{
			Name:     streamName,
			Subjects: []string{subject},
			Storage:  nats.FileStorage,
			MaxAge:   7 * 24 * time.Hour,
		})
		if err != nil {
			slog.Error("add stream", "error", err)
			os.Exit(1)
		}
		sub, err := js.Subscribe(subject, func(msg *nats.Msg) {
			if parseAndWrite(msg.Data, msg.Subject) {
				_ = msg.Ack()
			} else {
				_ = msg.Nak()
			}
		}, nats.Durable("questdb-bridge"), nats.ManualAck())
		if err != nil {
			slog.Error("subscribe", "error", err)
			os.Exit(1)
		}
		defer func() { _ = sub.Unsubscribe() }()
	}

	slog.Info("bridge running, ctrl-c to stop")
	<-ctx.Done()
	slog.Info("shutting down")
}
