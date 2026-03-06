package main

import (
	"os"
	"path/filepath"
	"testing"
)

func TestStripJSONComments_NoComments(t *testing.T) {
	in := []byte(`{"services": {}, "tui": {}}`)
	got := stripJSONComments(in)
	if string(got) != `{"services": {}, "tui": {}}` {
		t.Fatalf("unchanged input should stay same: got %q", got)
	}
}

func TestStripJSONComments_SingleLine(t *testing.T) {
	in := []byte(`{"services": {}, // comment
"tui": {}}`)
	got := stripJSONComments(in)
	want := `{"services": {}, 
"tui": {}}`
	if string(got) != want {
		t.Fatalf("strip //: got %q", got)
	}
}

func TestStripJSONComments_BlockComment(t *testing.T) {
	in := []byte(`{"services": {}, /* block */ "tui": {}}`)
	got := stripJSONComments(in)
	want := `{"services": {},  "tui": {}}`
	if string(got) != want {
		t.Fatalf("strip /* */: got %q", got)
	}
}

func TestStripJSONComments_StringWithDoubleSlash(t *testing.T) {
	in := []byte(`{"url": "https://example.com"}`)
	got := stripJSONComments(in)
	if string(got) != `{"url": "https://example.com"}` {
		t.Fatalf("// inside string must be preserved: got %q", got)
	}
}

func TestStripJSONComments_BlockAcrossLines(t *testing.T) {
	in := []byte("{\"x\": 1, /* a\nb */ \"y\": 2}")
	got := stripJSONComments(in)
	want := "{\"x\": 1,  \"y\": 2}"
	if string(got) != want {
		t.Fatalf("multiline block: got %q", got)
	}
}

func TestValidateConfig_ValidWithAllowedKey(t *testing.T) {
	dir := t.TempDir()
	path := filepath.Join(dir, "config.json")
	if err := os.WriteFile(path, []byte(`{"services": {}, "tui": {}}`), 0644); err != nil {
		t.Fatal(err)
	}
	if err := validateConfig(path); err != nil {
		t.Fatalf("expected valid: %v", err)
	}
}

func TestValidateConfig_ValidWithComments(t *testing.T) {
	dir := t.TempDir()
	path := filepath.Join(dir, "config.json")
	withComments := []byte(`{"services": {}, // example
"tui": {}}`)
	if err := os.WriteFile(path, withComments, 0644); err != nil {
		t.Fatal(err)
	}
	if err := validateConfig(path); err != nil {
		t.Fatalf("expected valid with comments: %v", err)
	}
}

func TestValidateConfig_InvalidJSON(t *testing.T) {
	dir := t.TempDir()
	path := filepath.Join(dir, "bad.json")
	if err := os.WriteFile(path, []byte(`{not valid}`), 0644); err != nil {
		t.Fatal(err)
	}
	if err := validateConfig(path); err == nil {
		t.Fatal("expected error for invalid JSON")
	}
}

func TestValidateConfig_NoAllowedKey(t *testing.T) {
	dir := t.TempDir()
	path := filepath.Join(dir, "config.json")
	if err := os.WriteFile(path, []byte(`{"other": 1}`), 0644); err != nil {
		t.Fatal(err)
	}
	if err := validateConfig(path); err == nil {
		t.Fatal("expected error when no allowed key")
	}
}

func TestValidateConfig_FileNotFound(t *testing.T) {
	if err := validateConfig("/nonexistent/path/config.json"); err == nil {
		t.Fatal("expected error for missing file")
	}
}

func TestValidateAPIContract_HasSection(t *testing.T) {
	dir := t.TempDir()
	path := filepath.Join(dir, "api.md")
	if err := os.WriteFile(path, []byte("# API\n\n## Endpoints\n\nSome text.\n"), 0644); err != nil {
		t.Fatal(err)
	}
	if err := validateAPIContract(path); err != nil {
		t.Fatalf("expected valid: %v", err)
	}
}

func TestValidateAPIContract_NoSection(t *testing.T) {
	dir := t.TempDir()
	path := filepath.Join(dir, "api.md")
	if err := os.WriteFile(path, []byte("# API\n\nNo double-hash section here.\n"), 0644); err != nil {
		t.Fatal(err)
	}
	if err := validateAPIContract(path); err == nil {
		t.Fatal("expected error when no ## section")
	}
}

func TestValidateAPIContract_FileNotFound(t *testing.T) {
	if err := validateAPIContract("/nonexistent/api.md"); err == nil {
		t.Fatal("expected error for missing file")
	}
}
