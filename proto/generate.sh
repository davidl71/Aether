#!/usr/bin/env bash
# Generate protobuf code for all languages.
# Run from repo root: ./proto/generate.sh
set -euo pipefail

PROTO_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(dirname "$PROTO_DIR")"

echo "=== Protobuf codegen ==="

# C++ (platform proto only; used by native/ with system protobuf)
CPP_OUT="$ROOT_DIR/native/generated"
if command -v protoc >/dev/null 2>&1; then
  echo "[c++] generating..."
  mkdir -p "$CPP_OUT"
  protoc -I "$PROTO_DIR" \
    --cpp_out="$CPP_OUT" \
    "$PROTO_DIR/messages.proto"
  echo "[c++] wrote $CPP_OUT/messages.pb.{h,cc} (use same protoc version as build)"
else
  echo "[c++] protoc not found, skipping"
fi

echo ""
PYTHON_OUT="$ROOT_DIR/python/generated"
mkdir -p "$PYTHON_OUT"
if command -v protoc >/dev/null 2>&1; then
  echo "[py] generating with betterproto..."
  python3 -m grpc_tools.protoc \
    -I "$PROTO_DIR" \
    --python_betterproto_out="$PYTHON_OUT" \
    "$PROTO_DIR/messages.proto" 2>/dev/null || \
  echo "[py] betterproto not installed, skipping (pip install betterproto[compiler])"
else
  echo "[py] protoc not found, skipping"
fi

# Go
GO_OUT="$ROOT_DIR/agents/go/proto/v1"
if command -v protoc-gen-go >/dev/null 2>&1; then
  mkdir -p "$GO_OUT"
  echo "[go] generating..."
  protoc -I "$PROTO_DIR" \
    --go_out="$GO_OUT" --go_opt=paths=source_relative \
    "$PROTO_DIR/messages.proto"
else
  echo "[go] protoc-gen-go not found, skipping (go install google.golang.org/protobuf/cmd/protoc-gen-go@latest)"
fi

# TypeScript (ts-proto)
TS_OUT="$ROOT_DIR/web/src/proto"
if command -v protoc-gen-ts_proto >/dev/null 2>&1 || [ -x "$ROOT_DIR/web/node_modules/.bin/protoc-gen-ts_proto" ]; then
  mkdir -p "$TS_OUT"
  echo "[ts] generating with ts-proto..."
  PLUGIN="${ROOT_DIR}/web/node_modules/.bin/protoc-gen-ts_proto"
  if [ ! -x "$PLUGIN" ]; then PLUGIN="protoc-gen-ts_proto"; fi
  protoc -I "$PROTO_DIR" \
    --plugin="protoc-gen-ts_proto=$PLUGIN" \
    --ts_proto_out="$TS_OUT" \
    --ts_proto_opt=esModuleInterop=true \
    "$PROTO_DIR/messages.proto" 2>/dev/null || \
  echo "[ts] ts-proto generation failed, skipping"
else
  echo "[ts] ts-proto not found, skipping (npm i -D ts-proto)"
fi

# Rust is handled by prost via build.rs (nats_adapter crate)
echo "[rs] codegen handled by prost build.rs"

echo "=== done ==="
