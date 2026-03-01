# Convenience Makefile: wraps CMake (presets) and scripts (lint, exarp-go).
# Override preset: make build PRESET=linux-x64-debug

ROOT := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))
SCRIPT := $(ROOT)scripts
export PROJECT_ROOT := $(ROOT)

# Default preset by OS (override with make PRESET=linux-x64-release, etc.)
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
  PRESET ?= macos-arm64-debug
else
  PRESET ?= linux-x64-debug
endif

.PHONY: all build configure test clean help
.PHONY: lint exarp-lint exarp-tool exarp-list exarp-testing exarp-security

# --- CMake (preset-based) ---
all build:
	cmake --build --preset $(PRESET)

configure:
	cmake --preset $(PRESET)

test:
	ctest --preset $(PRESET) --output-on-failure

clean:
	cmake --build --preset $(PRESET) --target clean

# --- Lint / exarp-go (scripts) ---
lint:
	$(SCRIPT)/run_linters.sh

exarp-lint:
	$(SCRIPT)/run_exarp_go_tool.sh lint

exarp-list:
	$(SCRIPT)/run_exarp_go_tool.sh --list

exarp-tool:
	$(SCRIPT)/run_exarp_go_tool.sh "$(if $(TOOL),$(TOOL),lint)"

exarp-testing:
	$(SCRIPT)/run_exarp_go_tool.sh testing

exarp-security:
	$(SCRIPT)/run_exarp_go_tool.sh security

help:
	@echo "CMake (preset=$(PRESET), override with PRESET=...):"
	@echo "  make all build   - cmake --build --preset $(PRESET)"
	@echo "  make configure   - cmake --preset $(PRESET)"
	@echo "  make test        - ctest --preset $(PRESET)"
	@echo "  make clean       - cmake --build --preset $(PRESET) --target clean"
	@echo ""
	@echo "Lint / exarp-go:"
	@echo "  make lint        - run_linters.sh (includes exarp-go when available)"
	@echo "  make exarp-lint  - exarp-go lint"
	@echo "  make exarp-list  - list exarp-go tools"
	@echo "  make exarp-tool TOOL=<name> - run tool (default: lint)"
	@echo "  make exarp-testing / make exarp-security"
