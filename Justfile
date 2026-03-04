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

# Configure CMake (one-time setup)
configure:
    cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Debug

# Build in debug mode
build:
    ninja -C build

# Build in release mode
build-release:
    cmake --build --preset macos-arm64-release

# Build universal binary (arm64 + x86_64)
build-universal:
    ./scripts/build_universal.sh

# Portable build: auto-detect macOS (Intel/ARM) or Linux and use matching CMake preset
# Usage: just build-portable [build|clean|test|install] [--debug|--release]
build-portable *args:
    ./scripts/build_portable.sh {{args}}

# Build TWS API shared library
build-twsapi:
    cmake -S native/ibapi_cmake -B native/ibapi_cmake/build -DCMAKE_BUILD_TYPE=Release
    cmake --build native/ibapi_cmake/build -j$(sysctl -n hw.ncpu 2>/dev/null || nproc)

# Build third-party dependencies (Intel Decimal + TWS API)
build-deps:
    just build-intel-decimal
    just build-twsapi 2>/dev/null || cmake --build native/ibapi_cmake/build --target all 2>/dev/null || echo "[info] TWS API built via main CMake"

# Clean all build artifacts
clean:
    rm -rf build/* cmake-build-*
    find . -name 'CMakeCache.txt' -delete
    find . -name 'CMakeFiles' -type d -exec rm -rf {} + 2>/dev/null || true

# --- Test ---

# Run all C++ tests
test:
    ctest --test-dir build --output-on-failure

# Run a specific test by name
test-one name:
    ctest --test-dir build -R {{name}} --output-on-failure

# Run Python tests
test-python:
    uv run pytest python/tests/ -v

# Run Python tests with coverage
test-python-cov:
    uv run pytest python/tests/ -v --cov

# --- Lint & Format ---

# Run all linters
lint:
    ./scripts/run_linters.sh

# Format C++ code with clang-format
format:
    find native/src native/include -name '*.cpp' -o -name '*.h' | xargs clang-format -i

# Run ESLint on web frontend
lint-web:
    cd web && npm run lint

# Run TypeScript type check
typecheck:
    cd web && npm run type-check

# Auto-fix all fixable issues (format + lint-fix)
fix:
    find native/src native/include -name '*.cpp' -o -name '*.h' | xargs clang-format -i
    cd web && npm run lint:fix 2>/dev/null || true
    cd web && npm run lint:css:fix 2>/dev/null || true
    @echo "All auto-fixable issues resolved"

# Pre-push checks (format, lint, test, build)
pre-push: format lint test build
    @echo "All pre-push checks passed — safe to push"

# Tag current commit as last known-good build
tag-ok: build test
    git tag -f build-ok
    @echo "Tagged current commit as build-ok"
    @echo "  Compare changes: git diff build-ok"
    @echo "  See breakage:    git log build-ok..HEAD --oneline"

# --- Run ---

# Run CLI (dry-run mode)
run:
    ./build/bin/ib_box_spread --dry-run

# Run CLI with config (dry-run mode)
run-config:
    ./build/bin/ib_box_spread --config config/config.json --dry-run

# Run Python TUI
run-tui:
    ./scripts/run_python_tui.sh

# --- Python ---

# Install Python dependencies
py-sync:
    uv sync

# Install a Python package
py-add package:
    uv pip install {{package}}

# --- Services ---

# Start a single service (run `just svc list` to see names)
svc action service="":
    ./scripts/service.sh {{action}} {{service}}

# Start all backend services
services-start:
    ./scripts/start_all_services.sh

# Stop all backend services
services-stop:
    ./scripts/stop_all_services.sh

# Start memcached (cache backend)
start-memcached:
    ./scripts/service.sh start memcached

# Stop memcached
stop-memcached:
    ./scripts/service.sh stop memcached

# --- Git ---

# Setup new git worktree
worktree:
    ./scripts/setup_worktree.sh

# --- Quality ---

# Generate project scorecard
scorecard:
    python3 python/tools/generate_project_scorecard.py

# Review code with local Ollama model
review-ollama *files:
    python3 python/tools/ollama_code_review.py --files {{files}} --model codellama:7b

# Review code with local MLX model
review-mlx *files:
    python3 python/tools/diffucode_review.py --files {{files}}

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
    mkdir -p python/generated
    protoc --proto_path=proto \
           --proto_path=native/third_party/tws-api/source/proto \
           --python_out=python/generated \
           proto/messages.proto 2>/dev/null || \
    (protoc --proto_path=proto --proto_path=native/third_party/tws-api/source/proto \
            --python_betterproto_out=python/generated/proto \
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
    @if command -v buf >/dev/null 2>&1; then \
      buf lint proto/; \
    else \
      echo "[warn] buf not installed (brew install bufbuild/buf/buf)"; \
      just proto-check; \
    fi

# --- Build Variants ---

# Fast incremental build with ccache/sccache
build-fast:
    ./scripts/build_variant.sh fast

# Distributed build with distcc + ccache
build-distributed:
    ./scripts/build_variant.sh distributed

# Build with timestamped log
build-logging preset="macos-arm64-debug":
    ./scripts/build_variant.sh logging {{preset}}

# Fast incremental build (parallel jobs, no cache)
build-fast-parallel:
    ninja -C build -j $(sysctl -n hw.ncpu 2>/dev/null || nproc)

# Build with ccache enabled
build-cached:
    cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Debug -DENABLE_CCACHE=ON && ninja -C build

# Verify C++ toolchain (Xcode CLT headers, cmake, ninja)
verify-toolchain:
    ./scripts/verify_toolchain.sh

# Build Intel Decimal math library
build-intel-decimal:
    cmake -S native/third_party/IntelRDFPMathLib20U2/LIBRARY -B native/third_party/IntelRDFPMathLib20U2/LIBRARY/build -DCMAKE_BUILD_TYPE=Release
    cmake --build native/third_party/IntelRDFPMathLib20U2/LIBRARY/build -j$(sysctl -n hw.ncpu 2>/dev/null || nproc)

# --- Info ---

# Show project info and build presets
info:
    @echo "Project: Multi-Asset Synthetic Financing Platform"
    @echo "Arch:    $(uname -m)"
    @echo "Build:   $(ls build/bin/ib_box_spread 2>/dev/null && echo 'ready' || echo 'not built')"
    @echo ""
    @cmake --list-presets 2>/dev/null | head -20 || echo "Run: just configure"

# Check TWS API setup
check-tws:
    ./scripts/check_tws_download.sh

# Validate config file
validate-config:
    ./build/bin/ib_box_spread --config config/config.json --validate
