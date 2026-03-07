// config-validator validates shared config JSON and optionally API contract markdown.
//
// Usage:
//
//	config-validator [flags]
//	-config path     Config file (default: IB_BOX_SPREAD_CONFIG or config/config.json)
//	-api-contract path  If set, validate API contract has required sections
//
// Exit: 0 on success, 1 on validation failure.
package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"log/slog"
	"os"
	"path/filepath"
	"strings"
)

func env(key, fallback string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return fallback
}

func main() {
	slog.SetDefault(slog.New(slog.NewJSONHandler(os.Stdout, nil)))
	configPath := flag.String("config", env("IB_BOX_SPREAD_CONFIG", "config/config.json"), "Path to config JSON")
	apiContractPath := flag.String("api-contract", "", "Path to API contract markdown (optional)")
	flag.Parse()

	ok := true

	// Validate config JSON
	if *configPath != "" {
		if err := validateConfig(*configPath); err != nil {
			slog.Error("config validation failed", "path", *configPath, "error", err)
			ok = false
		} else {
			slog.Info("config OK", "path", *configPath)
		}
	}

	// Validate API contract if path given
	if *apiContractPath != "" {
		if err := validateAPIContract(*apiContractPath); err != nil {
			slog.Error("api-contract validation failed", "path", *apiContractPath, "error", err)
			ok = false
		} else {
			slog.Info("api-contract OK", "path", *apiContractPath)
		}
	}

	if !ok {
		os.Exit(1)
	}
}

// validateConfig checks path exists, is valid JSON, and has at least one expected top-level key.
func validateConfig(path string) error {
	abs, err := filepath.Abs(path)
	if err != nil {
		return err
	}
	data, err := os.ReadFile(abs)
	if err != nil {
		return fmt.Errorf("read %s: %w", path, err)
	}

	// Strip single-line // comments so we can validate config.example.json-style files
	data = stripJSONComments(data)

	var m map[string]json.RawMessage
	if err := json.Unmarshal(data, &m); err != nil {
		return fmt.Errorf("invalid JSON: %w", err)
	}

	// Require at least one of these (shared config / C++ config)
	allowed := map[string]bool{
		"services": true, "tui": true, "tws": true, "strategy": true,
		"tastytrade": true, "alpaca": true, "version": true, "dataSources": true,
	}
	for k := range m {
		if allowed[k] {
			return nil
		}
	}
	return fmt.Errorf("config must contain at least one of: services, tui, tws, strategy, tastytrade, alpaca, version, dataSources")
}

func stripJSONComments(data []byte) []byte {
	s := string(data)
	var out strings.Builder
	i := 0
	inString := false
	var quote byte
	for i < len(s) {
		c := s[i]
		if inString {
			if c == '\\' && i+1 < len(s) {
				out.WriteByte(c)
				out.WriteByte(s[i+1])
				i += 2
				continue
			}
			if c == quote {
				inString = false
			}
			out.WriteByte(c)
			i++
			continue
		}
		if c == '"' {
			inString = true
			quote = '"'
			out.WriteByte(c)
			i++
			continue
		}
		if i+1 < len(s) && s[i:i+2] == "//" {
			for i < len(s) && s[i] != '\n' {
				i++
			}
			continue
		}
		if i+1 < len(s) && s[i:i+2] == "/*" {
			i += 2
			for i+1 < len(s) && s[i:i+2] != "*/" {
				i++
			}
			if i+1 < len(s) {
				i += 2
			}
			continue
		}
		out.WriteByte(c)
		i++
	}
	return []byte(out.String())
}

// validateAPIContract checks file exists and contains at least one ## section.
func validateAPIContract(path string) error {
	abs, err := filepath.Abs(path)
	if err != nil {
		return err
	}
	data, err := os.ReadFile(abs)
	if err != nil {
		return fmt.Errorf("read %s: %w", path, err)
	}
	s := string(data)
	if !strings.Contains(s, "## ") {
		return fmt.Errorf("%s: missing required section (## ...)", path)
	}
	return nil
}
