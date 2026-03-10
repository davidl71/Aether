package main

import (
	"encoding/base64"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	pbv1 "github.com/dlowes/ib-platform/agents/go/proto/v1"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/timestamppb"
)

func TestEnv_Fallback(t *testing.T) {
	got := env("UNLIKELY_ENV_VAR_12345", "default_val")
	if got != "default_val" {
		t.Fatalf("expected fallback, got %q", got)
	}
}

func TestEnv_Set(t *testing.T) {
	t.Setenv("TEST_GW_KEY", "custom")
	got := env("TEST_GW_KEY", "fallback")
	if got != "custom" {
		t.Fatalf("expected custom, got %q", got)
	}
}

func TestCorsMiddleware_Preflight(t *testing.T) {
	inner := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		t.Fatal("inner handler should not be called for OPTIONS")
	})
	handler := corsMiddleware(inner)

	req := httptest.NewRequest(http.MethodOptions, "/api/v1/snapshot", nil)
	rec := httptest.NewRecorder()
	handler.ServeHTTP(rec, req)

	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", rec.Code)
	}
	if rec.Header().Get("Access-Control-Allow-Origin") != "*" {
		t.Fatal("missing CORS header")
	}
}

func TestCorsMiddleware_PassThrough(t *testing.T) {
	inner := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	})
	handler := corsMiddleware(inner)

	req := httptest.NewRequest(http.MethodGet, "/", nil)
	rec := httptest.NewRecorder()
	handler.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	if rec.Header().Get("Access-Control-Allow-Origin") != "*" {
		t.Fatal("CORS header should be set on all responses")
	}
}

func TestGatewayHealthEndpoint(t *testing.T) {
	mux := http.NewServeMux()
	mux.HandleFunc("/gateway/health", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(map[string]string{"status": "ok"})
	})

	req := httptest.NewRequest(http.MethodGet, "/gateway/health", nil)
	rec := httptest.NewRecorder()
	mux.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}

	var body map[string]string
	if err := json.NewDecoder(rec.Body).Decode(&body); err != nil {
		t.Fatalf("decode: %v", err)
	}
	if body["status"] != "ok" {
		t.Fatalf("expected ok, got %q", body["status"])
	}
}

func TestNewRoute(t *testing.T) {
	r := newRoute("/api/", "http://localhost:8080")
	if r.prefix != "/api/" {
		t.Fatalf("prefix mismatch: %q", r.prefix)
	}
	if r.target.Host != "localhost:8080" {
		t.Fatalf("target host mismatch: %q", r.target.Host)
	}
	if r.proxy == nil {
		t.Fatal("proxy should not be nil")
	}
}

func TestDecodeEnvelopeMetadata(t *testing.T) {
	now := time.Now().UTC()
	env := &pbv1.NatsEnvelope{
		Id:          "msg-1",
		Source:      "collector",
		MessageType: "StrategyDecision",
		Payload:     []byte("abc"),
		Timestamp:   timestamppb.New(now),
	}
	data, err := proto.Marshal(env)
	if err != nil {
		t.Fatalf("marshal envelope: %v", err)
	}

	meta := decodeEnvelopeMetadata(data)
	if meta["id"] != "msg-1" {
		t.Fatalf("unexpected id: %#v", meta["id"])
	}
	if meta["message_type"] != "StrategyDecision" {
		t.Fatalf("unexpected message type: %#v", meta["message_type"])
	}
	if meta["payload_b64"] != base64.StdEncoding.EncodeToString([]byte("abc")) {
		t.Fatalf("unexpected payload_b64: %#v", meta["payload_b64"])
	}
	if meta["timestamp"] != now.Format(time.RFC3339Nano) {
		t.Fatalf("unexpected timestamp: %#v", meta["timestamp"])
	}
}
