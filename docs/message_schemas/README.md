# NATS Message Schemas

**DEPRECATED** -- JSON schema files have been replaced by
`proto/messages.proto` (protobuf), which is the canonical cross-language
contract.

## Single proto story

Platform messages live in **`proto/messages.proto`**. This is the single canonical cross-language contract — REST and NATS both use it. There is no gRPC server in this repo; do not add tonic/grpc dependencies.

TWS API vendor protos under `native/third_party/tws-api/` are Interactive Brokers upstream files built separately. They are not part of the platform contract.

## Where to find schemas

| Language   | Source                                           | Generated output | Status |
|------------|--------------------------------------------------|------------------|--------|
| Protobuf   | `proto/messages.proto`                           | (canonical source) | Canonical |
| C++        | CMake at build; or `./proto/generate.sh`         | `native/generated/` | Active |
| Rust       | `nats_adapter/build.rs` (prost, auto on `cargo build`) | `nats_adapter::proto::v1` (in-crate) | Active |
| Go         | `./proto/generate.sh`                            | `agents/go/proto/v1/` | Active |
| Python     | `./proto/generate.sh` (betterproto)              | `python/generated/` | Active |
| TypeScript | `./proto/generate.sh` (ts-proto; `cd web && npm i -D ts-proto` first) | `web/src/proto/` | Active |

From the `web/` directory run **`npm run generate:proto`** to regenerate TypeScript from `proto/messages.proto`.

> **Note on TWS API protos:** Proto files under `native/third_party/tws-api/` are
> vendor-only (Interactive Brokers upstream). They are built separately and are not
> part of the platform message contract. Do not import them from `proto/messages.proto`.

## Codegen

Run `./proto/generate.sh` from the repo root to regenerate all languages.

**Prerequisites:**
- **buf** (optional): single command for C++/Go/TS codegen; if missing, the script falls back to `protoc`. Installed automatically by **Ansible** (devtools role). Manual install: `brew install bufbuild/buf/buf` (macOS) or see [buf install](https://buf.build/docs/cli/installation).
- C++: `protoc` (system package)
- Python: `pip install betterproto[compiler]` (or `uv pip install betterproto[compiler]`)
- TypeScript: `cd web && npm i -D ts-proto`
- Go: `go install google.golang.org/protobuf/cmd/protoc-gen-go@latest`
- Rust: automatic via `nats_adapter/build.rs` during `cargo build`

**Python re-export:** `./proto/generate.sh` writes betterproto output into `python/generated/ib/platform/v1.py`. The directory `python/generated/` is gitignored. For a single import surface (`from python.generated import StrategySignal, DiscountBankBalance`, etc.), create or recreate `python/generated/__init__.py` that re-exports from `python.generated.ib.platform.v1`. After codegen, run from repo root:

```bash
uv run python scripts/recreate_python_generated_init.py
```

That script introspects the generated `v1` module and writes `python/generated/__init__.py` with all message/enum re-exports. If you add new messages to `proto/messages.proto`, run `./proto/generate.sh` then run the script again to refresh the re-exports.

## NATS message format

C++ publishes protobuf binary messages wrapped in `NatsEnvelope`:

```proto
message NatsEnvelope {
  string id = 1;
  google.protobuf.Timestamp timestamp = 2;
  string source = 3;
  string message_type = 4;
  bytes payload = 5;
}
```

Topics: `market-data.tick.<symbol>`, `strategy.signal.<symbol>`, `strategy.decision.<symbol>`

Payload bytes are a serialized inner message (`MarketDataEvent`, `StrategySignal`, or `StrategyDecision`).

## Migration status

| Component | JSON→Proto | Notes |
|-----------|-----------|-------|
| C++ NATS publish | Done | `nats_client.cpp` uses NatsEnvelope + protobuf |
| Rust NATS subscribe | Active | `nats_adapter` uses prost |
| Python NATS | Pending | Use generated types from `python/generated/` |
| TypeScript | Pending | Use generated types from `web/src/proto/` |

**Python boundary types:** All Python code at NATS/REST boundaries should use types from **`python/generated`** (generated from `proto/messages.proto` via `./proto/generate.sh`). The former `python/proto_types.py` is deprecated and has been removed; no callers remain. Import from `python.generated` (e.g. `from python.generated import StrategySignal, DiscountBankBalance`).

## Further reading

- [`proto/messages.proto`](../../proto/messages.proto) — canonical message definitions
- [`proto/generate.sh`](../../proto/generate.sh) — regenerate all language outputs
- [`scripts/recreate_python_generated_init.py`](../../scripts/recreate_python_generated_init.py) — recreate Python `python/generated/__init__.py` after codegen
- [`agents/backend/crates/nats_adapter/`](../../agents/backend/crates/nats_adapter/) — Rust prost codegen (build.rs)

**Planning (single proto story, dedup, execution order):**
- [PROTOBUF_DEDUP_OPPORTUNITIES.md](../planning/PROTOBUF_DEDUP_OPPORTUNITIES.md) — consolidation opportunities and TWS vendor protos
- [PROTOBUF_DEDUP_CONCRETE_PLAN.md](../planning/PROTOBUF_DEDUP_CONCRETE_PLAN.md) — concrete plan for proto cleanup
- Backlog and wave execution order: regenerate with `exarp-go -tool report -args '{"action":"plan"}'` (see `.cursor/plans/` when present)
