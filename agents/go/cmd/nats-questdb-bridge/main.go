// nats-questdb-bridge subscribes to NATS JetStream market data subjects
// and writes each tick to QuestDB via the ILP (InfluxDB Line Protocol) TCP
// ingestion endpoint.
//
// Environment:
//
//	NATS_URL          (default "nats://localhost:4222")
//	QUESTDB_ILP_ADDR  (default "localhost:9009")
//	NATS_STREAM       (default "MARKET_DATA")
//	NATS_SUBJECT      (default "market.data.>")
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

type tick struct {
	Symbol    string  `json:"symbol"`
	Bid       float64 `json:"bid"`
	Ask       float64 `json:"ask"`
	Last      float64 `json:"last"`
	Volume    uint64  `json:"volume"`
	Timestamp string  `json:"timestamp"`
}

func main() {
	natsURL := env("NATS_URL", nats.DefaultURL)
	questAddr := env("QUESTDB_ILP_ADDR", "localhost:9009")
	streamName := env("NATS_STREAM", "MARKET_DATA")
	subject := env("NATS_SUBJECT", "market.data.>")

	log.Printf("nats-questdb-bridge starting  nats=%s  questdb=%s  stream=%s  subject=%s",
		natsURL, questAddr, streamName, subject)

	nc, err := nats.Connect(natsURL,
		nats.RetryOnFailedConnect(true),
		nats.MaxReconnects(-1),
		nats.ReconnectWait(2*time.Second),
	)
	if err != nil {
		log.Fatalf("nats connect: %v", err)
	}
	defer nc.Close()

	js, err := nc.JetStream()
	if err != nil {
		log.Fatalf("jetstream: %v", err)
	}

	// Ensure stream exists (idempotent)
	_, err = js.AddStream(&nats.StreamConfig{
		Name:     streamName,
		Subjects: []string{subject},
		Storage:  nats.FileStorage,
		MaxAge:   7 * 24 * time.Hour,
	})
	if err != nil {
		log.Fatalf("add stream: %v", err)
	}

	// Connect to QuestDB ILP endpoint
	conn, err := net.DialTimeout("tcp", questAddr, 5*time.Second)
	if err != nil {
		log.Fatalf("questdb connect: %v", err)
	}
	defer conn.Close()

	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()

	sub, err := js.Subscribe(subject, func(msg *nats.Msg) {
		var t tick
		if err := json.Unmarshal(msg.Data, &t); err != nil {
			log.Printf("unmarshal: %v", err)
			_ = msg.Nak()
			return
		}

		ts := time.Now()
		if t.Timestamp != "" {
			if parsed, e := time.Parse(time.RFC3339Nano, t.Timestamp); e == nil {
				ts = parsed
			}
		}

		// ILP line: market_data,symbol=SPY bid=450.1,ask=450.2,last=450.15,volume=12345i <ns>
		sym := strings.ReplaceAll(t.Symbol, " ", "\\ ")
		line := fmt.Sprintf("market_data,symbol=%s bid=%.6f,ask=%.6f,last=%.6f,volume=%di %d\n",
			sym, t.Bid, t.Ask, t.Last, t.Volume, ts.UnixNano())

		if _, err := conn.Write([]byte(line)); err != nil {
			log.Printf("questdb write: %v", err)
			_ = msg.Nak()
			return
		}

		_ = msg.Ack()
	}, nats.Durable("questdb-bridge"), nats.ManualAck())
	if err != nil {
		log.Fatalf("subscribe: %v", err)
	}
	defer func() { _ = sub.Unsubscribe() }()

	log.Println("bridge running, ctrl-c to stop")
	<-ctx.Done()
	log.Println("shutting down")
}
