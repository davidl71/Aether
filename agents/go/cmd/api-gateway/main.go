// api-gateway is a lightweight reverse proxy built entirely with the Go
// standard library.  It routes requests to the appropriate backend
// service based on URL prefix and provides CORS, logging, and a
// combined /health endpoint.
//
// Environment:
//
//	LISTEN_ADDR       (default ":9000")
//	BACKEND_URL       (default "http://localhost:8080")
//	TRADIER_URL       (default "http://localhost:8006")
//	HEARTBEAT_URL     (default "http://localhost:8090")
package main

import (
	"context"
	"encoding/json"
	"log"
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
		log.Fatalf("bad url for %s: %v", prefix, err)
	}
	return route{
		prefix: prefix,
		target: u,
		proxy:  httputil.NewSingleHostReverseProxy(u),
	}
}

func main() {
	listenAddr := env("LISTEN_ADDR", ":9000")
	backendURL := env("BACKEND_URL", "http://localhost:8080")
	tradierURL := env("TRADIER_URL", "http://localhost:8006")
	heartbeatURL := env("HEARTBEAT_URL", "http://localhost:8090")

	routes := []route{
		newRoute("/api/v1/tradier/", tradierURL),
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
		log.Printf("api-gateway listening on %s", listenAddr)
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Fatal(err)
		}
	}()

	<-ctx.Done()
	shutCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	_ = srv.Shutdown(shutCtx)
	log.Println("api-gateway stopped")
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
		log.Printf("%s %s %s", r.Method, r.URL.Path, time.Since(start).Round(time.Millisecond))
	})
}
