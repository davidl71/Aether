# Multi-Asset Synthetic Financing Platform
# Run `just --list` to see all available commands
#
# Nix: run any recipe inside the Nix dev shell with `just nix <recipe> [args...]`
# Example: just nix build  or  just nix test-python

# Default recipe: show available commands
default:
    @just --list --unsorted

# --- Nix ---

# Run a just recipe inside the Nix dev shell (cmake, ninja, uv, cargo from flake)
# Usage: just nix build  |  just nix test  |  just nix test-python  etc.
nix *args:
    nix develop . --extra-experimental-features "nix-command flakes" -c just {{args}}

# --- Build ---
# Primary codebase is Rust (agents/backend). C++ native build removed; CMake at root is for lint/convenience only.

# Configure CMake (optional; used for lint targets and scripts that invoke cmake)
configure:
    cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Debug
    @echo "Note: native C++ build removed; use 'just build' or 'just build-rust' for Rust."

# Build in debug mode (Rust workspace; primary)
build:
    just build-rust

# Build in release mode (Rust)
build-release:
    cd agents/backend && cargo build --release

# AI-friendly Rust build: quiet, log to file, emit JSON result. For tools/CI.
build-ai-friendly:
    ./scripts/build_rust_ai_friendly.sh

# AI-friendly Rust build, JSON only to stdout (for piping to jq/tools)
build-ai-friendly-json:
    ./scripts/build_rust_ai_friendly.sh --json-only

# Clean CMake/build dirs (optional). Primary clean for Rust: just clean-rust
clean:
    rm -rf build/* cmake-build-*
    find . -name 'CMakeCache.txt' -delete
    find . -name 'CMakeFiles' -type d -exec rm -rf {} + 2>/dev/null || true

# Clean Rust build artifacts (agents/backend target/ or CARGO_TARGET_DIR). Global cache: ~/.cargo/registry, ~/.cargo/git.
clean-rust:
    cd agents/backend && cargo clean
    @echo "Rust workspace cleaned. To free more: rm -rf ~/.cargo/registry/cache ~/.cargo/git (global cache)."

# Save cargo-sweep timestamp (run after a successful build so sweep removes only older artifacts).
# Requires: cargo install cargo-sweep. Automated: run after successful `just test` and `just build-rust`.
sweep-stamp:
    cd agents/backend && cargo sweep sweep . --stamp

# Remove Cargo target artifacts older than the last sweep-stamp. Use after builds to free disk; next build will recompile removed crates.
# CI runs sweep (agents-backend-rust.yml) after build/test to keep the target cache small.
# Dry-run: just sweep-dry
sweep:
    cd agents/backend && cargo sweep sweep . --file .
sweep-dry:
    cd agents/backend && cargo sweep sweep . --file . --dry-run

# Prune artifacts older than N days (default 14). For cron/daily: 0 2 * * * cd /path/to/repo && just sweep-auto
# Requires: cargo install cargo-sweep
sweep-auto days="14":
    cd agents/backend && cargo sweep sweep . --time {{days}}

# Find unused dependencies (compile-time check, more accurate than cargo-machete). Requires: cargo install cargo-udeps, rustup install nightly
udeps:
    cd agents/backend && rustup run nightly cargo udeps

# Show Cargo global cache size (~/.cargo). Requires: cargo install cargo-cache
cache:
    cargo cache
# Trim global cache to limit (default 500M). Dry-run: just cache-trim-dry [limit]
cache-trim limit="500M":
    cargo cache trim --limit {{limit}}
cache-trim-dry limit="1G":
    cargo cache trim --dry-run --limit {{limit}}
# Remove source checkouts (frees space; crates re-download on next build)
cache-autoclean:
    cargo cache -a

# Build Rust workspace (debug). Uses the workspace rustc-wrapper; if sccache is
# installed it is picked up automatically. On success, updates cargo-sweep stamp
# if installed.
build-rust:
    cd agents/backend && env SCCACHE_DIR="${SCCACHE_DIR:-$HOME/.cache/sccache}" cargo build && (command -v cargo-sweep >/dev/null 2>&1 && cargo sweep sweep . --stamp || true)

# --- Test ---

# Run Rust tests (primary; C++ tests removed with native build). Uses the
# workspace rustc-wrapper; if sccache is installed it is picked up
# automatically. On success, updates cargo-sweep stamp if installed.
test:
    cd agents/backend && env SCCACHE_DIR="${SCCACHE_DIR:-$HOME/.cache/sccache}" cargo test && (command -v cargo-sweep >/dev/null 2>&1 && cargo sweep sweep . --stamp || true)

# Run a specific Rust test by name (e.g. just test-one risk_calculator)
test-one name:
    cd agents/backend && env SCCACHE_DIR="${SCCACHE_DIR:-$HOME/.cache/sccache}" cargo test {{name}} && (command -v cargo-sweep >/dev/null 2>&1 && cargo sweep sweep . --stamp || true)

# Run Python tests (nautilus agent)
test-python:
    cd agents/nautilus && uv run pytest tests/ -v

# Run Python tests with coverage (nautilus)
test-python-cov:
    cd agents/nautilus && uv run pytest tests/ -v --cov=nautilus_agent --cov-report=term-missing

# Run TUI E2E tests (@microsoft/tui-test; requires Node 18+)
test-tui-e2e:
    cd tui-e2e && npm ci && npm test

# --- Lint & Format ---

# Run all linters
lint:
    ./scripts/run_linters.sh

# Run linters in parallel (independent linters concurrently; exarp + shellcheck after)
lint-parallel:
    ./scripts/run_linters.sh --parallel

# Run linters in AI-friendly mode (quiet, log to logs/, emit JSON)
lint-ai-friendly:
    ./scripts/run_linters.sh --ai-friendly

# Run linters, print only JSON to stdout (for tools/AI)
lint-ai-friendly-json:
    ./scripts/run_linters.sh --json-only

# Run shellcheck on scripts and Ansible run script (exarp-go pattern)
lint-shell:
    @command -v shellcheck >/dev/null 2>&1 || (echo "shellcheck not found (brew install shellcheck / apt install shellcheck)" && exit 1)
    shellcheck -x scripts/*.sh ansible/run-dev-setup.sh
    @echo "lint-shell done"

# CMake lint (cmake-lint from cmakelang). Root CMakeLists only; native/ removed. Requires: pip install cmakelang or uv tool install cmakelang
lint-cmake:
    @command -v cmake-lint >/dev/null 2>&1 || (echo "cmake-lint not found (pip install cmakelang or uv tool install cmakelang)" && exit 1)
    cmake-lint CMakeLists.txt
    @echo "lint-cmake done"

# Shell-only lint, single JSON line to stdout (for tools/AI). Log to logs/lint_shell_ai.log.
lint-shell-ai:
    ./scripts/lint_shell_ai.sh

# Python-only lint (ruff + bandit). Full lint is `just lint`. native/tests/python removed with native build.
lint-python:
    uv run --with ruff ruff check scripts
    @command -v bandit >/dev/null 2>&1 && bandit -r scripts || echo "[skip] bandit not installed (optional)"
    @echo "lint-python done"

# Show lint log paths and tail main log (logs/lint_ai_friendly.log). Creates logs when you run lint --ai-friendly.
# Usage: just lint-log [N]  (N = lines to tail, default 60);  just lint-log --list  |  just lint-log --all
lint-log *args:
    ./scripts/check_lint_logs.sh {{args}}

# Format C++ code with clang-format (no-op when native/ removed; kept for reference)
format:
    @test -d native/src && find native/src native/include -name '*.cpp' -o -name '*.h' 2>/dev/null | xargs clang-format -i || echo "[skip] no native/src (C++ build removed)"

# Run ESLint on web frontend
lint-web:
    cd web && npm run lint

# Run TypeScript type check
typecheck:
    cd web && npm run type-check

# Auto-fix all fixable issues (format + lint-fix). C++ format skipped when native/ removed.
fix:
    @test -d native/src && find native/src native/include -name '*.cpp' -o -name '*.h' 2>/dev/null | xargs clang-format -i || true
    cd web && npm run lint:fix 2>/dev/null || true
    cd web && npm run lint:css:fix 2>/dev/null || true
    @echo "All auto-fixable issues resolved"

# Pre-push checks (format, lint, test, build). format is no-op when native/ absent.
pre-push: format lint test build-rust
    @echo "All pre-push checks passed — safe to push"

# Lighter pre-commit checks (format + lint only; no test/build)
pre-commit: format lint
    @echo "Pre-commit checks passed — format and lint OK"

# Pull with uncommitted changes (stash → pull → pop)
pull-safe:
    ./scripts/git_pull_safe.sh

# Tag current commit as last known-good build (Rust build + test)
tag-ok: build-rust test
    git tag -f build-ok
    @echo "Tagged current commit as build-ok"
    @echo "  Compare changes: git diff build-ok"
    @echo "  See breakage:    git log build-ok..HEAD --oneline"

# --- Run ---

# Run Rust CLI (from agents/backend)
run:
    cd agents/backend && cargo run -p cli --

# Run Rust CLI with config (TOML; see config/config.toml)
run-config:
    cd agents/backend && cargo run -p cli -- --config ../config/config.toml

# Run Rust TUI (NATS-only; start NATS and backend_service for live data)
run-tui:
    ./scripts/run_rust_tui.sh

# Run TWS yield curve daemon (standalone: TWS → NATS KV yield_curve.{symbol}). Requires NATS + TWS. Env: TWS_PORT, SYMBOLS, INTERVAL_SECS.
run-tws-yield-daemon:
    cd agents/backend && cargo run -p tws_yield_curve_daemon

# Verify CLI box yield curve report (snapshot-write + yield-curve --symbol SPX). Requires NATS and backend_service running.
check-yield:
    ./scripts/check_snapshot_yield_curve.sh

# --- Credentials ---
# Secure credential management using config file (~/.config/aether/) + keyring fallback
# Usage: just cred-set <name> <value> | just cred-get <name> | just cred-delete <name>

# Set a named credential (see `just cred-list` for the full set)
cred-set name value:
    @cd agents/backend && cargo run -p cli -- cred set {{name}} {{value}}

# Set FRED API key (prompts for value)
cred-set-fred:
    @cd agents/backend && cargo run -p cli -- cred set fred

# Set FMP API key
cred-set-fmp:
    @cd agents/backend && cargo run -p cli -- cred set fmp

# Set Polygon API key
cred-set-polygon:
    @cd agents/backend && cargo run -p cli -- cred set polygon

# Set Alpaca paper credentials
cred-set-alpaca-paper:
    @cd agents/backend && cargo run -p cli -- cred set alpaca-paper-key && cargo run -p cli -- cred set alpaca-paper-secret

# Set Alpaca live credentials
cred-set-alpaca-live:
    @cd agents/backend && cargo run -p cli -- cred set alpaca-live-key && cargo run -p cli -- cred set alpaca-live-secret

# Set both Alpaca paper and live credentials
cred-set-alpaca:
    @just cred-set-alpaca-paper
    @just cred-set-alpaca-live

# Set Tastytrade credentials
cred-set-tastytrade:
    @cd agents/backend && cargo run -p cli -- cred set tastytrade-key && cargo run -p cli -- cred set tastytrade-account

# Get a credential value (shows masked output)
cred-get name:
    @cd agents/backend && cargo run -p cli -- cred get {{name}}

# Delete a stored credential
cred-delete name:
    @cd agents/backend && cargo run -p cli -- cred delete {{name}}

# List available credentials
cred-list:
    @cd agents/backend && cargo run -p cli -- cred list

# Check which credentials are configured
cred-status:
    cd agents/backend && cargo run -q -p cli -- cred list 2>/dev/null

# Sanity check: Python binding tests + Rust TUI buildability. On success, updates cargo-sweep stamp if installed.
sanity:
    just test-python
    cd agents/backend && env $(command -v sccache >/dev/null 2>&1 && echo RUSTC_WRAPPER=sccache) SCCACHE_DIR="${SCCACHE_DIR:-../.cache/sccache}" cargo check -p tui_service && (command -v cargo-sweep >/dev/null 2>&1 && cargo sweep sweep . --stamp || true)

# Install a Python package
py-add package:
    uv pip install {{package}}

# --- Services ---
# Service scripts: scripts/service.sh (unified), scripts/service_manager.sh (rust_backend + nats).
# See scripts/SERVICE_MANAGER_README.md. Order: nats → rust → memcached → gateway → web.

# Start a single service (run `just svc list` to see: nats, rust, memcached, gateway, web)
svc action service="":
    ./scripts/service.sh {{action}} {{service}}

# Start all backend services (nats, rust, then optional memcached/gateway/web per config)
services-start:
    ./scripts/start_all_services.sh

# Stop all backend services (reverse order)
services-stop:
    ./scripts/stop_all_services.sh

# Restart all backend services
services-restart:
    ./scripts/restart_all_services.sh

# Status of all backend services
services-status:
    ./scripts/status_all_services.sh

# Minimal active stack: start only rust_backend + nats (scripts/service_manager.sh)
services-active-start:
    ./scripts/service_manager.sh start-all

# Stop minimal active stack (rust_backend + nats)
services-active-stop:
    ./scripts/service_manager.sh stop-all

# Status of minimal active stack
services-active-status:
    ./scripts/service_manager.sh status

# Start memcached (cache backend)
start-memcached:
    ./scripts/service.sh start memcached

# Stop memcached
stop-memcached:
    ./scripts/service.sh stop memcached

# --- Git ---

# Add a new git worktree (script removed; use git directly)
# Usage: just worktree <path> [branch]
worktree path branch="":
    if [ -n "{{branch}}" ]; then git worktree add "{{path}}" "{{branch}}"; else git worktree add "{{path}}"; fi

# --- Quality ---

# List exarp-go tools (requires exarp-go on PATH or EXARP_GO_ROOT)
exarp-list:
    ./scripts/run_exarp_go.sh -list -quiet

# Show project backlog: task counts, next actions, and overview (runs exarp report, writes docs/PROJECT_OVERVIEW.md)
exarp-backlog:
    ./scripts/run_exarp_go.sh -tool report -quiet

# Run exarp-go tool (default: lint). Usage: just exarp lint | just exarp testing | just exarp security
exarp tool="lint":
    ./scripts/run_exarp_go.sh -tool {{tool}} -quiet

# Run exarp-go lint only (default: Go linter only; no args)
exarp-lint:
    ./scripts/run_exarp_go.sh -tool lint -quiet

# Run exarp-go lint with shellcheck on scripts/ (Go + shell when used with exarp-lint)
exarp-lint-shell:
    ./scripts/run_exarp_go.sh -tool lint -args '{"linter":"shellcheck","path":"scripts"}' -quiet

# Generate project scorecard
scorecard:
    ./scripts/run_exarp_go.sh -tool report -quiet

# --- Protobuf ---

# Generate protobuf code for all languages (C++, Python, Rust, TypeScript)
proto-gen:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "[proto] Generating C++ from IBKR protos..."
    mkdir -p native/generated/proto
    protoc --proto_path=native/third_party/tws-api/source/proto \
           --cpp_out=native/generated/proto \
           native/third_party/tws-api/source/proto/*.proto
    echo "[proto] Generating C++ from platform protos..."
    protoc --proto_path=proto \
           --proto_path=native/third_party/tws-api/source/proto \
           --cpp_out=native/generated/proto \
           proto/messages.proto
    echo "[proto] Generating Python..."
    mkdir -p native/generated/python
    protoc --proto_path=proto \
           --proto_path=native/third_party/tws-api/source/proto \
           --python_out=native/generated/python \
           proto/messages.proto 2>/dev/null || \
    (protoc --proto_path=proto --proto_path=native/third_party/tws-api/source/proto \
            --python_betterproto_out=native/generated/python/proto \
            proto/messages.proto 2>/dev/null || echo "[warn] Python codegen failed")
    echo "[proto] Generating Rust (prost)..."
    echo "  (handled by agents/build.rs via prost-build)"
    echo "[proto] Generating TypeScript (ts-proto)..."
    mkdir -p web/src/generated/proto
    TS_PROTO_PLUGIN="./web/node_modules/.bin/protoc-gen-ts_proto"
    [ -x "$TS_PROTO_PLUGIN" ] || TS_PROTO_PLUGIN="$(which protoc-gen-ts_proto 2>/dev/null || true)"
    if [ -n "$TS_PROTO_PLUGIN" ]; then \
      protoc --proto_path=proto \
             --proto_path=native/third_party/tws-api/source/proto \
             --plugin="protoc-gen-ts_proto=$TS_PROTO_PLUGIN" \
             --ts_proto_out=web/src/generated/proto \
             proto/messages.proto 2>/dev/null || echo "[warn] ts-proto generation failed"; \
    else \
      echo "[warn] ts-proto plugin not found (cd web && npm install; or run ansible playbook for global install)"; \
    fi
    echo "[proto] Done."

# Check protobuf files for syntax errors
proto-check:
    protoc --proto_path=proto \
           --proto_path=native/third_party/tws-api/source/proto \
           --descriptor_set_out=/dev/null \
           proto/messages.proto
    @echo "[proto] Syntax OK"

# Lint protobuf files with buf (if installed)
proto-lint:
    ./scripts/buf_lint_and_breaking.sh --lint-only

# Breaking-change check: proto/ vs main branch
proto-breaking:
    ./scripts/buf_lint_and_breaking.sh --breaking-only

# Lint + breaking (run both; use in CI)
proto-validate:
    ./scripts/buf_lint_and_breaking.sh

# --- Build Variants ---
# Legacy C++ variants removed. Use 'just build' or 'just build-rust' (sccache used when available).

# Verify Rust toolchain (rustc, cargo)
verify-rust:
    rustc --version && cargo --version
    @echo "Rust toolchain OK"

# Verify C++/CMake toolchain (optional; for scripts that still use cmake/ninja)
verify-toolchain:
    ./scripts/verify_toolchain.sh

# Ansible development setup (install deps, syntax-check, run playbook; uses SSL fix on macOS)
ansible-dev:
    ./ansible/run-dev-setup.sh

# Ansible dev setup without sudo prompt (for AI/automated runs; macOS needs no sudo; Debian tasks that need sudo will be skipped/fail)
ansible-dev-no-sudo:
    ANSIBLE_NO_BECOME=1 ./ansible/run-dev-setup.sh

# Ansible playbook syntax-check only (quick validation)
ansible-check:
    cd ansible && ansible-playbook --syntax-check -i inventories/development playbooks/development.yml
    @echo "Ansible syntax OK"

# Ansible-lint (playbooks and roles). Requires: pip install ansible-lint or uv tool install ansible-lint
ansible-lint:
    ansible-lint ansible/
    @echo "ansible-lint done"

# --- Info ---

# Show project info (Rust primary)
info:
    @echo "Project: Multi-Asset Synthetic Financing Platform (Rust-first)"
    @echo "Arch:    $(uname -m)"
    @(test -d agents/backend/target/debug && echo "Rust:    built (debug)" || echo "Rust:    not built")
    @echo "Commands: just build | just test | just run | just run-tui"
    @cmake --list-presets 2>/dev/null | head -10 || true

# Check TWS API / IBKR gateway setup (optional; script may require native/third_party)
check-tws:
    ./scripts/check_tws_download.sh

# Benchmark backend services (health + snapshot latency)
benchmark:
    uv run python scripts/benchmark_backend_services.py

# --- NautilusTrader Agent ---

# Install/sync NautilusTrader agent dependencies
nautilus-sync:
    cd agents/nautilus && uv sync

# Generate Python protobuf stubs for the nautilus agent (requires grpcio-tools in venv)
proto-gen-nautilus:
    cd agents/nautilus && uv run python scripts/generate_proto.py

# Start NautilusTrader IB agent in paper trading mode (port 7497)
nautilus-paper:
    cd agents/nautilus && uv run python -m nautilus_agent.main config/default.toml

# Start NautilusTrader IB agent with custom config path
nautilus-start config="agents/nautilus/config/default.toml":
    cd agents/nautilus && uv run python -m nautilus_agent.main {{config}}

# Run NautilusTrader agent unit tests
test-nautilus:
    cd agents/nautilus && uv run pytest tests/ -v
