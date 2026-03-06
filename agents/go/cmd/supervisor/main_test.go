package main

import (
	"encoding/json"
	"path/filepath"
	"testing"
	"time"
)

func TestEnv_Fallback(t *testing.T) {
	got := env("UNLIKELY_ENV_VAR_77777", "default_cfg")
	if got != "default_cfg" {
		t.Fatalf("expected default_cfg, got %q", got)
	}
}

func TestServiceSpecParsing(t *testing.T) {
	input := `[
		{"name": "backend", "cmd": ["cargo", "run"], "dir": "agents/backend"},
		{"name": "web", "cmd": ["npm", "run", "dev"], "dir": "web"}
	]`

	var specs []serviceSpec
	if err := json.Unmarshal([]byte(input), &specs); err != nil {
		t.Fatalf("unmarshal: %v", err)
	}

	if len(specs) != 2 {
		t.Fatalf("expected 2 specs, got %d", len(specs))
	}
	if specs[0].Name != "backend" {
		t.Fatalf("expected backend, got %q", specs[0].Name)
	}
	if specs[0].Dir != "agents/backend" {
		t.Fatalf("expected agents/backend, got %q", specs[0].Dir)
	}
	if len(specs[0].Cmd) != 2 {
		t.Fatalf("expected 2 cmd parts, got %d", len(specs[0].Cmd))
	}
	if specs[1].Name != "web" {
		t.Fatalf("expected web, got %q", specs[1].Name)
	}
}

func TestServiceSpecParsing_Empty(t *testing.T) {
	var specs []serviceSpec
	if err := json.Unmarshal([]byte("[]"), &specs); err != nil {
		t.Fatalf("unmarshal: %v", err)
	}
	if len(specs) != 0 {
		t.Fatalf("expected 0, got %d", len(specs))
	}
}

func TestMin(t *testing.T) {
	tests := []struct {
		a, b, want time.Duration
	}{
		{1 * time.Second, 30 * time.Second, 1 * time.Second},
		{30 * time.Second, 1 * time.Second, 1 * time.Second},
		{5 * time.Second, 5 * time.Second, 5 * time.Second},
	}
	for _, tc := range tests {
		got := min(tc.a, tc.b)
		if got != tc.want {
			t.Errorf("min(%v, %v) = %v, want %v", tc.a, tc.b, got, tc.want)
		}
	}
}

func TestResolveWorkDir(t *testing.T) {
	absDir := t.TempDir() // always absolute
	got := resolveWorkDir("/project", absDir)
	if got != absDir {
		t.Errorf("absolute dir should be unchanged: got %q", got)
	}

	got = resolveWorkDir("/project", "agents/backend")
	want := filepath.Join("/project", "agents/backend")
	if got != want {
		t.Errorf("relative: got %q want %q", got, want)
	}
}
