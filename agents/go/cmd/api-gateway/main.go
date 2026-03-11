// api-gateway is a lightweight reverse proxy built entirely with the Go
// standard library.  It routes requests to the appropriate backend
// service based on URL prefix and provides CORS, logging, and a
// combined /health endpoint.
//
// TUI can use this as an optional specialist-routing entry point. The
// default shared frontend origin is Rust (:8080); gateway now keeps only
// the remaining explicit specialist and operational routes.
//
// Environment:
//
//	LISTEN_ADDR        (default ":9000")
//	BACKEND_URL        (default "http://localhost:8080") — Rust
//	HEARTBEAT_URL      (default "http://localhost:8090")
package main

import (
	"context"
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

func main() {
	slog.SetDefault(slog.New(slog.NewJSONHandler(os.Stdout, nil)))
	listenAddr := env("LISTEN_ADDR", ":9000")
	backendURL := env("BACKEND_URL", "http://localhost:8080")
	heartbeatURL := env("HEARTBEAT_URL", "http://localhost:8090")

	// Order matters: more specific prefixes first. Only heartbeat and Rust remain here.
	routes := []route{
		newRoute("/api/heartbeat/", heartbeatURL),
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
