// api-gateway is a lightweight reverse proxy built entirely with the Go
// standard library.  It routes requests to the appropriate backend
// service based on URL prefix and provides CORS, logging, and a
// combined /health endpoint.
//
// TUI and Web use this as single entry point (P1-B): TUI RestProvider
// presets point at gateway; gateway proxies to Rust (:8080) or Python
// backends (:8000, :8002, etc.) with path rewrite.
//
// Environment:
//
//	LISTEN_ADDR        (default ":9000")
//	BACKEND_URL        (default "http://localhost:8080") — Rust
//	TRADIER_URL        (default "http://localhost:8006")
//	HEARTBEAT_URL      (default "http://localhost:8090")
//	IB_URL             (default "http://localhost:8002") — IB REST snapshot
//	ALPACA_URL         (default "http://localhost:8000")
//	TRADESTATION_URL   (default "http://localhost:8001")
//	TASTYTRADE_URL     (default "http://localhost:8005")
//	NATS_URL           optional; if set, gateway reads live state from NATS KV (bucket LIVE_STATE) at /api/live/state and /api/live/state/watch (SSE)
package main

import (
	"context"
	"encoding/base64"
	"encoding/json"
	"log/slog"
	"net/http"
	"net/http/httputil"
	"net/url"
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

type route struct {
	prefix string
	target *url.URL
	proxy  *httputil.ReverseProxy
}

func newRoute(prefix, rawURL string) route {
	u, err := url.Parse(rawURL)
	if err != nil {
		slog.Error("bad url for route", "prefix", prefix, "error", err)
		os.Exit(1)
	}
	return route{
		prefix: prefix,
		target: u,
		proxy:  httputil.NewSingleHostReverseProxy(u),
	}
}

// newRouteWithStrip proxies prefix to target but rewrites path: strip stripPrefix
// and replace with replacePrefix. E.g. prefix=/api/v1/ib/, stripPrefix=/api/v1/ib, replacePrefix=/api/v1
// so GET /api/v1/ib/snapshot -> GET http://target/api/v1/snapshot
func newRouteWithStrip(prefix, stripPrefix, replacePrefix, rawURL string) route {
	u, err := url.Parse(rawURL)
	if err != nil {
		slog.Error("bad url for route", "prefix", prefix, "error", err)
		os.Exit(1)
	}
	proxy := httputil.NewSingleHostReverseProxy(u)
	proxy.Director = func(r *http.Request) {
		r.URL.Scheme = u.Scheme
		r.URL.Host = u.Host
		if strings.HasPrefix(r.URL.Path, stripPrefix) {
			r.URL.Path = replacePrefix + r.URL.Path[len(stripPrefix):]
		}
		if r.URL.RawPath != "" && strings.HasPrefix(r.URL.RawPath, stripPrefix) {
			r.URL.RawPath = replacePrefix + r.URL.RawPath[len(stripPrefix):]
		}
	}
	return route{prefix: prefix, target: u, proxy: proxy}
}

func main() {
	slog.SetDefault(slog.New(slog.NewJSONHandler(os.Stdout, nil)))
	listenAddr := env("LISTEN_ADDR", ":9000")
	backendURL := env("BACKEND_URL", "http://localhost:8080")
	tradierURL := env("TRADIER_URL", "http://localhost:8006")
	heartbeatURL := env("HEARTBEAT_URL", "http://localhost:8090")
	ibURL := env("IB_URL", "http://localhost:8002")
	alpacaURL := env("ALPACA_URL", "http://localhost:8000")
	tradestationURL := env("TRADESTATION_URL", "http://localhost:8001")
	tastytradeURL := env("TASTYTRADE_URL", "http://localhost:8005")

	// Order matters: more specific prefixes first. TUI presets use /api/v1/{ib,alpaca,...}/snapshot.
	routes := []route{
		newRoute("/api/v1/tradier/", tradierURL),
		newRoute("/api/heartbeat/", heartbeatURL),
		newRouteWithStrip("/api/v1/ib/", "/api/v1/ib", "/api/v1", ibURL),
		newRouteWithStrip("/api/v1/alpaca/", "/api/v1/alpaca", "/api", alpacaURL),
		newRouteWithStrip("/api/v1/tradestation/", "/api/v1/tradestation", "/api", tradestationURL),
		newRouteWithStrip("/api/v1/tastytrade/", "/api/v1/tastytrade", "/api/v1", tastytradeURL),
		newRoute("/api/", backendURL),
		newRoute("/health", backendURL),
	}

	mux := http.NewServeMux()

	mux.HandleFunc("/gateway/health", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(map[string]string{
			"status": "ok",
			"as_of":  time.Now().Format(time.RFC3339),
		})
	})

	var kv nats.KeyValue
	if natsURL := strings.TrimSpace(env("NATS_URL", "")); natsURL != "" {
		nc, err := nats.Connect(natsURL, nats.RetryOnFailedConnect(true), nats.MaxReconnects(3))
		if err != nil {
			slog.Warn("nats connect failed, /api/live/state disabled", "error", err)
		} else {
			js, err := nc.JetStream()
			if err != nil {
				slog.Warn("jetstream failed, /api/live/state disabled", "error", err)
			} else {
				kv, err = js.KeyValue("LIVE_STATE")
				if err != nil {
					slog.Warn("kv bucket LIVE_STATE not found, /api/live/state disabled", "error", err)
				} else {
					mux.HandleFunc("/api/live/state", func(w http.ResponseWriter, r *http.Request) {
						if r.Method != http.MethodGet {
							http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
							return
						}
						w.Header().Set("Content-Type", "application/json")
						keyQ := r.URL.Query().Get("key")
						if keyQ != "" {
							entry, err := kv.Get(keyQ)
							if err != nil {
								http.Error(w, `{"error":"key not found"}`, http.StatusNotFound)
								return
							}
							_ = json.NewEncoder(w).Encode(map[string]any{
								"key":      entry.Key(),
								"revision": entry.Revision(),
								"value":    base64.StdEncoding.EncodeToString(entry.Value()),
							})
							return
						}
						keys, err := kv.Keys()
						if err != nil {
							http.Error(w, `{"error":"list keys failed"}`, http.StatusInternalServerError)
							return
						}
						_ = json.NewEncoder(w).Encode(map[string]any{"keys": keys})
					})
					mux.HandleFunc("/api/live/state/watch", func(w http.ResponseWriter, r *http.Request) {
						if r.Method != http.MethodGet {
							http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
							return
						}
						watcher, err := kv.Watch(">")
						if err != nil {
							http.Error(w, `{"error":"watch failed"}`, http.StatusInternalServerError)
							return
						}
						defer func() { _ = watcher.Stop() }()
						w.Header().Set("Content-Type", "text/event-stream")
						w.Header().Set("Cache-Control", "no-cache")
						w.Header().Set("Connection", "keep-alive")
						w.Header().Set("X-Accel-Buffering", "no")
						if flusher, ok := w.(http.Flusher); ok {
							flusher.Flush()
						}
						updates := watcher.Updates()
						for {
							select {
							case <-r.Context().Done():
								return
							case entry, ok := <-updates:
								if !ok {
									return
								}
								if entry == nil {
									// Initial values done (nil sentinel)
									if _, err := w.Write([]byte("event: synced\ndata: {}\n\n")); err != nil {
										return
									}
								} else {
									payload := map[string]any{
										"key":      entry.Key(),
										"revision": entry.Revision(),
										"value":    base64.StdEncoding.EncodeToString(entry.Value()),
									}
									body, _ := json.Marshal(payload)
									if _, err := w.Write([]byte("data: " + string(body) + "\n\n")); err != nil {
										return
									}
								}
								if flusher, ok := w.(http.Flusher); ok {
									flusher.Flush()
								}
							}
						}
					})
					slog.Info("nats kv ready", "bucket", "LIVE_STATE")
				}
			}
		}
	}
	if kv == nil {
		mux.HandleFunc("/api/live/state", func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			w.WriteHeader(http.StatusServiceUnavailable)
			_ = json.NewEncoder(w).Encode(map[string]string{"error": "NATS KV not configured (set NATS_URL)"})
		})
		mux.HandleFunc("/api/live/state/watch", func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			w.WriteHeader(http.StatusServiceUnavailable)
			_ = json.NewEncoder(w).Encode(map[string]string{"error": "NATS KV not configured (set NATS_URL)"})
		})
	}

	mux.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		for _, rt := range routes {
			if strings.HasPrefix(r.URL.Path, rt.prefix) {
				rt.proxy.ServeHTTP(w, r)
				return
			}
		}
		http.NotFound(w, r)
	})

	handler := corsMiddleware(loggingMiddleware(mux))

	srv := &http.Server{
		Addr:         listenAddr,
		Handler:      handler,
		ReadTimeout:  15 * time.Second,
		WriteTimeout: 30 * time.Second,
		IdleTimeout:  60 * time.Second,
	}

	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()

	go func() {
		slog.Info("api-gateway listening", "addr", listenAddr)
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			slog.Error("server error", "error", err)
			os.Exit(1)
		}
	}()

	<-ctx.Done()
	shutCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	_ = srv.Shutdown(shutCtx)
	slog.Info("api-gateway stopped")
}

func corsMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Access-Control-Allow-Origin", "*")
		w.Header().Set("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
		w.Header().Set("Access-Control-Allow-Headers", "Content-Type, Authorization")
		if r.Method == http.MethodOptions {
			w.WriteHeader(http.StatusNoContent)
			return
		}
		next.ServeHTTP(w, r)
	})
}

func loggingMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		start := time.Now()
		next.ServeHTTP(w, r)
		slog.Info("request", "method", r.Method, "path", r.URL.Path, "duration", time.Since(start).Round(time.Millisecond))
	})
}
