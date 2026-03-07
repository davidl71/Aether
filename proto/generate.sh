#!/usr/bin/env bash
# Generate protobuf code for all languages.
# Run from repo root: ./proto/generate.sh
set -euo pipefail

PROTO_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(dirname "$PROTO_DIR")"

echo "=== Protobuf codegen ==="

# C++, Go, TypeScript via buf generate
if command -v buf >/dev/null 2>&1; then
  echo "[buf] running buf generate..."
  (cd "$PROTO_DIR" && buf generate)
  echo "[buf] done"
else
  echo "[buf] buf not found — falling back to raw protoc"

  # C++ fallback
  if command -v protoc >/dev/null 2>&1; then
    echo "[c++] generating..."
    mkdir -p "$ROOT_DIR/native/generated"
    protoc -I "$PROTO_DIR" --cpp_out="$ROOT_DIR/native/generated" "$PROTO_DIR/messages.proto"
  fi

  # Go fallback
  GO_OUT="$ROOT_DIR/agents/go/proto/v1"
  if command -v protoc-gen-go >/dev/null 2>&1; then
    mkdir -p "$GO_OUT"
    echo "[go] generating..."
    protoc -I "$PROTO_DIR" --go_out="$GO_OUT" --go_opt=paths=source_relative "$PROTO_DIR/messages.proto"
  fi

  # TypeScript fallback
  TS_OUT="$ROOT_DIR/web/src/proto"
  PLUGIN="${ROOT_DIR}/web/node_modules/.bin/protoc-gen-ts_proto"
  if [ -x "$PLUGIN" ] || command -v protoc-gen-ts_proto >/dev/null 2>&1; then
    mkdir -p "$TS_OUT"
    if [ ! -x "$PLUGIN" ]; then PLUGIN="protoc-gen-ts_proto"; fi
    echo "[ts] generating..."
    protoc -I "$PROTO_DIR" --plugin="protoc-gen-ts_proto=$PLUGIN" \
      --ts_proto_out="$TS_OUT" --ts_proto_opt=esModuleInterop=true \
      "$PROTO_DIR/messages.proto" 2>/dev/null || echo "[ts] ts-proto failed, skipping"
  fi
fi

# Python/betterproto (non-standard plugin — kept here, not in buf.gen.yaml)
echo ""
PYTHON_OUT="$ROOT_DIR/python/generated"
mkdir -p "$PYTHON_OUT"
if command -v uv >/dev/null 2>&1; then
  echo "[py] generating with betterproto..."
  uv run python3 -m grpc_tools.protoc \
    -I "$PROTO_DIR" \
    --python_betterproto_out="$PYTHON_OUT" \
    "$PROTO_DIR/messages.proto" 2>/dev/null || \
  echo "[py] betterproto generation failed (uv pip install betterproto[compiler])"
else
  python3 -m grpc_tools.protoc \
    -I "$PROTO_DIR" \
    --python_betterproto_out="$PYTHON_OUT" \
    "$PROTO_DIR/messages.proto" 2>/dev/null || \
  echo "[py] betterproto not installed, skipping"
fi

# Rust is handled by prost via build.rs (nats_adapter crate)
echo "[rs] codegen handled by prost build.rs"

echo "=== done ==="
