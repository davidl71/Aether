# NATS Message Schemas

**DEPRECATED** -- JSON schema files have been replaced by
`proto/messages.proto` (protobuf), which is the canonical cross-language
contract.

## Single proto story

Platform messages live in **`proto/messages.proto`**. This is the single canonical cross-language contract — REST and NATS both use it. There is no gRPC server in this repo; do not add tonic/grpc dependencies.

TWS API vendor protos (if present under `native/third_party/tws-api/`) are Interactive Brokers upstream files built separately. They are not part of the platform contract.

## Where to find schemas

| Language   | Source                                           | Generated output | Status |
|------------|--------------------------------------------------|------------------|--------|
| Protobuf   | `proto/messages.proto`                           | (canonical source) | Canonical |
| Rust       | `nats_adapter/build.rs` (prost, auto on `cargo build`) | `nats_adapter::proto::v1` (in-crate) | Active |
| Go         | `./proto/generate.sh`                            | `agents/go/proto/v1/` | Active |
| Python     | `./proto/generate.sh` (betterproto)              | `python/generated/` | Helper output |
| TypeScript | `./proto/generate.sh` (ts-proto; `cd web && npm i -D ts-proto` first) | `web/src/generated/proto/` | Generated; web archived |

From the `web/` directory run **`npm run generate:proto`** to regenerate TypeScript from `proto/messages.proto`.

> **Note on TWS API protos:** Proto files under `native/third_party/tws-api/` are
> vendor-only (Interactive Brokers upstream). They are built separately and are not
> part of the platform message contract. Do not import them from `proto/messages.proto`.

## Codegen

Run `./proto/generate.sh` from the repo root to regenerate all languages.

**Lint and breaking changes:** Run `./scripts/buf_lint_and_breaking.sh` (or `just proto-lint`, `just proto-breaking`, `just proto-validate`) to run buf lint and buf breaking against `main`. CI runs this on every push/PR. See `proto/buf.yaml` for lint/breaking rules.

**Breaking exception:** We `except: [ENUM_VALUE_SAME_NAME]` so enum value *renames* (e.g. `OPTION_TYPE_UNSPECIFIED` → `OPTION_TYPE_ENUM_UNSPECIFIED` for lint) are allowed. Wire format is unchanged (same numbers); only generated enum names change. All other FILE breaking rules (deletions, field/enum/message changes, etc.) remain in effect. See [Buf breaking rules](https://buf.build/docs/breaking/rules/#enum_value_same_name).

**Prerequisites:**

- **buf** (optional): single command for Go/TS codegen; if missing, the script falls back to `protoc`. Manual install: `brew install bufbuild/buf/buf` (macOS) or see [buf install](https://buf.build/docs/cli/installation).
- Python: `pip install betterproto[compiler]` (or `uv pip install betterproto[compiler]`)
- TypeScript: `cd web && npm i -D ts-proto`
- Go: `go install google.golang.org/protobuf/cmd/protoc-gen-go@latest`
- Rust: automatic via `nats_adapter/build.rs` during `cargo build`

**Python re-export:** `./proto/generate.sh` writes betterproto output into `python/generated/ib/platform/v1.py`. After codegen, run from repo root:

```bash
uv run python scripts/recreate_python_generated_init.py
```

That script writes `python/generated/__init__.py` with re-exports from the generated `v1` module. If you add new messages to `proto/messages.proto`, run `./proto/generate.sh` then run the script again to refresh the re-exports.

## NATS message format

The Rust backend (and any other publishers) send protobuf binary messages wrapped in `NatsEnvelope`:

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
| Rust NATS publish/subscribe | Active | Backend and `nats_adapter` use NatsEnvelope + prost |
| Python | Helper output | Use generated types from `python/generated/` when needed |
| TypeScript | Generated; web archived | `web/src/generated/proto/` |

**Python boundary types:** Use types from **`python/generated/`** (generated from `proto/messages.proto` via `./proto/generate.sh`; re-export script: `scripts/recreate_python_generated_init.py`).

## Further reading

- [`docs/platform/PROTO_OPPORTUNITIES_AND_BUF_CONFIG.md`](../platform/PROTO_OPPORTUNITIES_AND_BUF_CONFIG.md) — where else to use proto, optional buf config (WIRE_JSON, enum_zero_value_suffix)
- [`proto/messages.proto`](../../proto/messages.proto) — canonical message definitions (includes `Loan`, `LoanType`, `LoanStatus`, `LoansResponse` for Loans API binary responses)
- [`proto/generate.sh`](../../proto/generate.sh) — regenerate all language outputs
- [`scripts/recreate_python_generated_init.py`](../../scripts/recreate_python_generated_init.py) — recreate Python `python/generated/__init__.py` after codegen
- [`agents/backend/crates/nats_adapter/`](../../agents/backend/crates/nats_adapter/) — Rust prost codegen (build.rs)

**Planning (single proto story, dedup, execution order):**

- [PROTOBUF_DEDUP_OPPORTUNITIES.md](../planning/PROTOBUF_DEDUP_OPPORTUNITIES.md) — consolidation opportunities and TWS vendor protos
- [PROTOBUF_DEDUP_CONCRETE_PLAN.md](../planning/PROTOBUF_DEDUP_CONCRETE_PLAN.md) — concrete plan for proto cleanup
- Backlog and wave execution order: regenerate with `exarp-go -tool report -args '{"action":"plan"}'` (see `.cursor/plans/` when present)
