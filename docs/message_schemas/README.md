# NATS Message Schemas

**DEPRECATED** -- JSON schema files have been replaced by
`proto/messages.proto` (protobuf), which is the canonical cross-language
contract.

## Single proto story

Platform messages live in **`proto/messages.proto`**. TWS API vendor protos (when using GitHub layout) are built separately via `native/ibapi_cmake`. There is no gRPC server or backend proto in this repo; REST and NATS use the platform proto only.

## Where to find schemas

| Language   | Source                                           | Status |
|------------|--------------------------------------------------|--------|
| Protobuf   | `proto/messages.proto`                           | Canonical |
| C++        | Generated at build by CMake (`native/generated/`); or run `./proto/generate.sh` | Active |
| Rust       | `nats_adapter::proto::v1` (prost via `nats_adapter/build.rs`) | Active |
| Go         | `agents/go/proto/v1/messages.pb.go` (run `proto/generate.sh`) | Active |
| Python     | `python/generated/` (betterproto codegen, run `proto/generate.sh`) | Active |
| TypeScript | `web/src/proto/messages.ts` (ts-proto codegen, run `proto/generate.sh` after `npm i -D ts-proto`) | Active |

## Codegen

Run `./proto/generate.sh` from the repo root to regenerate all languages.

**Prerequisites:**
- C++: `protoc` (system package)
- Python: `pip install betterproto[compiler]` (or `uv pip install betterproto[compiler]`)
- TypeScript: `cd web && npm i -D ts-proto`
- Go: `go install google.golang.org/protobuf/cmd/protoc-gen-go@latest`
- Rust: automatic via `nats_adapter/build.rs` during `cargo build`

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

## Further reading

- [awesome-grpc](https://github.com/grpc-ecosystem/awesome-grpc) — Curated gRPC/protobuf ecosystem
