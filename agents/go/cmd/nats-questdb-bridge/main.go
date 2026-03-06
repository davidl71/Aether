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
	"log"
	"net"
	"os"
	"os/signal"
	"strings"
	"syscall"
	"time"

	"github.com/nats-io/nats.go"
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

func main() {
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
	log.Printf("nats-questdb-bridge starting  mode=%s  nats=%s  questdb=%s  subject=%s",
		mode, natsURL, questAddr, subject)

	nc, err := nats.Connect(natsURL,
		nats.RetryOnFailedConnect(true),
		nats.MaxReconnects(-1),
		nats.ReconnectWait(2*time.Second),
	)
	if err != nil {
		log.Fatalf("nats connect: %v", err)
	}
	defer nc.Close()

	conn, err := net.DialTimeout("tcp", questAddr, 5*time.Second)
	if err != nil {
		log.Fatalf("questdb connect: %v", err)
	}
	defer func() { _ = conn.Close() }()

	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()

	writeTick := func(t *tick, ts time.Time) {
		sym := strings.ReplaceAll(t.Symbol, " ", "\\ ")
		line := fmt.Sprintf("market_data,symbol=%s bid=%.6f,ask=%.6f,last=%.6f,volume=%di %d\n",
			sym, t.Bid, t.Ask, t.Last, t.Volume, ts.UnixNano())
		if _, err := conn.Write([]byte(line)); err != nil {
			log.Printf("questdb write: %v", err)
		}
	}

	parseAndWrite := func(data []byte, subj string) bool {
		tick, ts, err := parseTickPayload(data, subj)
		if err != nil {
			log.Printf("parse: %v", err)
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
			log.Fatalf("subscribe: %v", err)
		}
		defer func() { _ = sub.Unsubscribe() }()
	} else {
		streamName := env("NATS_STREAM", "MARKET_DATA")
		js, err := nc.JetStream()
		if err != nil {
			log.Fatalf("jetstream: %v", err)
		}
		_, err = js.AddStream(&nats.StreamConfig{
			Name:     streamName,
			Subjects: []string{subject},
			Storage:  nats.FileStorage,
			MaxAge:   7 * 24 * time.Hour,
		})
		if err != nil {
			log.Fatalf("add stream: %v", err)
		}
		sub, err := js.Subscribe(subject, func(msg *nats.Msg) {
			if parseAndWrite(msg.Data, msg.Subject) {
				_ = msg.Ack()
			} else {
				_ = msg.Nak()
			}
		}, nats.Durable("questdb-bridge"), nats.ManualAck())
		if err != nil {
			log.Fatalf("subscribe: %v", err)
		}
		defer func() { _ = sub.Unsubscribe() }()
	}

	log.Println("bridge running, ctrl-c to stop")
	<-ctx.Done()
	log.Println("shutting down")
}
