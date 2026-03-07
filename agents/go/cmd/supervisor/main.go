// supervisor manages child processes defined in a simple JSON config
// and restarts them on crash.  It replaces the proliferation of
// individual start_*.sh / stop_*.sh scripts with a single binary.
//
// Config: SUPERVISOR_CONFIG (default "config/services.json")
// Root for relative "dir": SUPERVISOR_ROOT (default "."). Set to project root when run from agents/go.
//
//	[
//	  {"name": "backend",  "cmd": ["cargo", "run", ...], "dir": "agents/backend"},
//	  {"name": "web",      "cmd": ["npm", "run", "dev"], "dir": "web"}
//	]
//
// Signals: SIGINT / SIGTERM stop all children and exit.
package main

import (
	"context"
	"encoding/json"
	"log/slog"
	"os"
	"os/exec"
	"os/signal"
	"path/filepath"
	"sync"
	"syscall"
	"time"
)

func env(key, fallback string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return fallback
}

type serviceSpec struct {
	Name string   `json:"name"`
	Cmd  []string `json:"cmd"`
	Dir  string   `json:"dir"`
}

// resolveWorkDir returns dir if absolute, otherwise filepath.Join(root, dir).
func resolveWorkDir(root, dir string) string {
	if filepath.IsAbs(dir) {
		return dir
	}
	return filepath.Join(root, dir)
}

func main() {
	slog.SetDefault(slog.New(slog.NewJSONHandler(os.Stdout, nil)))
	cfgPath := env("SUPERVISOR_CONFIG", "config/services.json")

	data, err := os.ReadFile(cfgPath)
	if err != nil {
		slog.Error("read config", "path", cfgPath, "error", err)
		os.Exit(1)
	}

	var specs []serviceSpec
	if err := json.Unmarshal(data, &specs); err != nil {
		slog.Error("parse config", "error", err)
		os.Exit(1)
	}

	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()

	var wg sync.WaitGroup
	for i := range specs {
		s := specs[i]
		wg.Add(1)
		go func() {
			defer wg.Done()
			supervise(ctx, s)
		}()
	}

	<-ctx.Done()
	slog.Info("shutting down all services")
	wg.Wait()
	slog.Info("all services stopped")
}

func supervise(ctx context.Context, s serviceSpec) {
	workDir := resolveWorkDir(env("SUPERVISOR_ROOT", "."), s.Dir)
	backoff := 1 * time.Second
	for {
		select {
		case <-ctx.Done():
			return
		default:
		}

		slog.Info("starting service", "name", s.Name, "cmd", s.Cmd, "dir", workDir)

		cmd := exec.CommandContext(ctx, s.Cmd[0], s.Cmd[1:]...)
		cmd.Dir = workDir
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr

		if err := cmd.Run(); err != nil {
			if ctx.Err() != nil {
				return
			}
			slog.Info("service exited with error, restarting", "name", s.Name, "error", err, "backoff", backoff)
			select {
			case <-time.After(backoff):
			case <-ctx.Done():
				return
			}
			backoff = min(backoff*2, 30*time.Second)
		} else {
			slog.Info("service exited cleanly", "name", s.Name)
			return
		}
	}
}

func min(a, b time.Duration) time.Duration {
	if a < b {
		return a
	}
	return b
}
