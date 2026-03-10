package main

import (
	"errors"
	"strings"
	"testing"

	pbv1 "github.com/dlowes/ib-platform/agents/go/proto/v1"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/timestamppb"
)

type stubSink struct {
	name   string
	called int
	err    error
}

func (s *stubSink) Write(_ *collectedMessage) error {
	s.called++
	return s.err
}

func (s *stubSink) Name() string { return s.name }

func TestKvKeyUsesMessageTypeAndSymbol(t *testing.T) {
	if got := kvKey("market-data.tick.SPY", "MarketDataEvent"); got != "MarketDataEvent.SPY" {
		t.Fatalf("unexpected key: %s", got)
	}
}

func TestEnvelopeSummaryIncludesEnvelopeMetadata(t *testing.T) {
	env := &pbv1.NatsEnvelope{
		Id:          "evt-1",
		Source:      "cpp-engine",
		MessageType: "MarketDataEvent",
		Payload:     []byte{0x01, 0x02, 0x03},
		Timestamp:   timestamppb.Now(),
	}
	data, err := proto.Marshal(env)
	if err != nil {
		t.Fatalf("marshal envelope: %v", err)
	}

	summary := envelopeSummary(data)
	if summary["id"] != "evt-1" {
		t.Fatalf("unexpected id: %#v", summary["id"])
	}
	if summary["message_type"] != "MarketDataEvent" {
		t.Fatalf("unexpected message type: %#v", summary["message_type"])
	}
	if summary["payload_len"] != 3 {
		t.Fatalf("unexpected payload len: %#v", summary["payload_len"])
	}
}

func TestWriteToSinksContinuesAfterSinkError(t *testing.T) {
	msg := &collectedMessage{
		Subject:     "market-data.tick.SPY",
		EnvelopeRaw: []byte{1, 2, 3},
		Envelope:    &pbv1.NatsEnvelope{MessageType: "MarketDataEvent"},
		MessageType: "MarketDataEvent",
	}
	fail := &stubSink{name: "fail", err: errors.New("boom")}
	ok := &stubSink{name: "ok"}
	m := newMetrics()

	err := writeToSinks(msg, []messageSink{fail, ok}, m)

	if fail.called != 1 {
		t.Fatalf("expected failing sink to be called once, got %d", fail.called)
	}
	if ok.called != 1 {
		t.Fatalf("expected succeeding sink to be called once, got %d", ok.called)
	}
	if m.writeErrors.Load() != 1 {
		t.Fatalf("expected one write error, got %d", m.writeErrors.Load())
	}
	if err == nil {
		t.Fatal("expected sink error to be returned")
	}
}

func TestDurableNameSanitizesSubject(t *testing.T) {
	got := durableName("collector", "market-data.tick.>")
	if got != "collector-market-data_tick" {
		t.Fatalf("unexpected durable name: %s", got)
	}
}

func TestMarketDataILPLineUsesEnvelopePayload(t *testing.T) {
	ts := timestamppb.Now()
	payload, err := proto.Marshal(&pbv1.MarketDataEvent{
		Symbol: "SPY",
		Bid:    100.25,
		Ask:    100.75,
		Last:   100.50,
		Volume: 42,
	})
	if err != nil {
		t.Fatalf("marshal payload: %v", err)
	}
	msg := &collectedMessage{
		Subject: "market-data.tick.SPY",
		Envelope: &pbv1.NatsEnvelope{
			MessageType: "MarketDataEvent",
			Payload:     payload,
			Timestamp:   ts,
		},
		MessageType: "MarketDataEvent",
	}

	line, err := marketDataILPLine(msg)
	if err != nil {
		t.Fatalf("marketDataILPLine: %v", err)
	}
	if !strings.Contains(line, "market_data,symbol=SPY") {
		t.Fatalf("unexpected line: %s", line)
	}
	if !strings.Contains(line, "bid=100.250000,ask=100.750000,last=100.500000,volume=42i") {
		t.Fatalf("unexpected metric fields: %s", line)
	}
}
