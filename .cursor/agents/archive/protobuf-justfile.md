# Agent D: Protobuf Codegen Justfile

## Role

Add Justfile recipes for protobuf code generation across all project languages, and a CI-friendly check recipe to verify generated code is up to date.

## Tasks

1. **Add Justfile recipe for protobuf codegen** (`T-1772135684402673000`)
   - Add a `# --- Protobuf ---` section to `Justfile` (after the `# --- Quality ---` section)
   - Add recipe `proto-gen` that runs codegen for all languages:
     - **Rust**: Already handled by `build.rs` in `agents/backend/crates/nats_adapter/` -- just note it
     - **Python**: `protoc --python_betterproto_out=python/generated/ proto/messages.proto` (or `grpcio-tools`)
     - **TypeScript**: `protoc --plugin=protoc-gen-ts_proto --ts_proto_out=web/src/generated/ proto/messages.proto`
     - **C++**: `protoc --cpp_out=native/generated/ proto/messages.proto`
     - Include IBKR proto import path: `--proto_path=native/third_party/tws-api/source/proto/`
   - Add recipe `proto-check` that:
     - Runs `proto-gen` into a temp directory
     - Diffs against committed generated code
     - Exits non-zero if out of date (CI-friendly)
   - Add recipe `proto-lint` that runs `buf lint proto/` (if buf is available)
   - Create output directories if they don't exist: `python/generated/`, `web/src/generated/`, `native/generated/`

## Proto File Locations

| File | Purpose |
|------|---------|
| `proto/messages.proto` | Our custom platform messages (package `ib.platform.v1`) |
| `native/third_party/tws-api/source/proto/*.proto` | 203 IBKR official proto files (package `protobuf`) |

## Justfile Section to Add

Add after the existing `# --- Quality ---` section, before `# --- Info ---`:

```just
# --- Protobuf ---

# Generate protobuf code for all languages
proto-gen:
    @echo "Generating protobuf code..."
    mkdir -p python/generated web/src/generated native/generated
    protoc \
        --proto_path=proto/ \
        --proto_path=native/third_party/tws-api/source/proto/ \
        --python_betterproto_out=python/generated/ \
        proto/messages.proto
    protoc \
        --proto_path=proto/ \
        --proto_path=native/third_party/tws-api/source/proto/ \
        --plugin=protoc-gen-ts_proto=$(which protoc-gen-ts_proto) \
        --ts_proto_out=web/src/generated/ \
        proto/messages.proto
    protoc \
        --proto_path=proto/ \
        --proto_path=native/third_party/tws-api/source/proto/ \
        --cpp_out=native/generated/ \
        proto/messages.proto
    @echo "Protobuf generation complete"

# Check that generated protobuf code is up to date (CI)
proto-check:
    #!/usr/bin/env bash
    set -euo pipefail
    tmpdir=$(mktemp -d)
    trap "rm -rf $tmpdir" EXIT
    # ... generate into tmpdir and diff
    echo "Protobuf check passed"
```

## Files You Own (exclusive)

- `Justfile` -- ONLY the `# --- Protobuf ---` section with `proto-gen`, `proto-check`, `proto-lint` recipes
- `python/generated/` (create directory)
- `web/src/generated/` (create directory)
- `native/generated/` (create directory)

## Files You Must NOT Touch

- `proto/messages.proto` (schema changes are a separate task)
- `native/src/` or `native/include/` (owned by Agent E)
- `scripts/` (owned by Agent B)
- `ansible/` (owned by Agent C)
- `Justfile` service/build recipes (owned by Agents A, B)

## Completion Criteria

- [ ] `just proto-gen` recipe exists and runs without error (may warn about missing plugins)
- [ ] `just proto-check` recipe exists and exits 0 when code is up to date
- [ ] Output directories created: `python/generated/`, `web/src/generated/`, `native/generated/`
- [ ] Exarp task `T-1772135684402673000` marked Done
