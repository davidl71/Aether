#!/usr/bin/env python3
"""Generate Python protobuf stubs for the nautilus agent.

Run from repo root:
    python agents/nautilus/scripts/generate_proto.py

Or via just:
    just proto-gen-nautilus

Requires grpcio-tools (included in dev dependencies):
    uv pip install grpcio-tools
"""

import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).parents[3]
PROTO_FILE = REPO_ROOT / "proto" / "messages.proto"
PROTO_DIR = REPO_ROOT / "proto"
OUT_DIR = REPO_ROOT / "agents" / "nautilus" / "src" / "nautilus_agent" / "generated"

OUT_DIR.mkdir(parents=True, exist_ok=True)

result = subprocess.run(
    [
        sys.executable,
        "-m", "grpc_tools.protoc",
        f"-I{PROTO_DIR}",
        f"--python_out={OUT_DIR}",
        str(PROTO_FILE),
    ],
    check=False,
)

if result.returncode != 0:
    print("ERROR: protoc failed. Install grpcio-tools: uv pip install grpcio-tools", file=sys.stderr)
    sys.exit(1)

print(f"Generated: {OUT_DIR}/messages_pb2.py")
